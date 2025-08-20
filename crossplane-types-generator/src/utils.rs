use std::{collections::HashMap, ffi::OsStr, path::Path, sync::LazyLock};

use anyhow::Context;

static CRDS: LazyLock<Option<&OsStr>> = LazyLock::new(|| Some(OsStr::new("crds")));
static YAML: LazyLock<Option<&OsStr>> = LazyLock::new(|| Some(OsStr::new("yaml")));
static PACKAGE: LazyLock<Option<&OsStr>> = LazyLock::new(|| Some(OsStr::new("package")));

pub static WORKSPACE_MANIFEST: LazyLock<crate::WorkspaceManifest> = LazyLock::new(|| {
    crate::CargoManifest::from_path_with_metadata(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/..",
        "/Cargo.toml",
    ))
    .expect("failed to read workspace `Cargo.toml` manifest")
    .workspace
    .expect("workspace `Cargo.toml` manifest missing required `[workspace]` table")
});

pub static WORKSPACE_PACKAGE: LazyLock<&'static cargo_toml::PackageTemplate> =
    LazyLock::new(|| {
        WORKSPACE_MANIFEST
            .package
            .as_ref()
            .expect("workspace `Cargo.toml` manifest missing required `[workspace.package]` table")
    });

pub static WORKSPACE_LINTS: LazyLock<HashMap<&'static str, &'static cargo_toml::LintGroups>> =
    LazyLock::new(|| HashMap::from_iter([("lints", &WORKSPACE_MANIFEST.lints)]));

#[track_caller]
pub fn to_toml(value: impl serde::Serialize) -> String {
    toml::to_string(&value).expect("failed to serialize value")
}

#[track_caller]
pub fn replace(string: &str, target: &str, replacement: &str) -> String {
    string.replace(target, replacement)
}

fn is_crd_filepath(file: &&str) -> bool {
    let filepath = <str as AsRef<Path>>::as_ref(file);

    let Some((parent, grandparent)) = filepath
        .parent()
        .and_then(|parent| Some(parent).zip(parent.parent()))
    else {
        return false;
    };

    // Check that:
    //   - extension is ".yaml"
    //   - the parent directory is "crds"
    //   - the parent-parent directory is "package"
    filepath.extension() == *YAML
        && parent.file_name() == *CRDS
        && grandparent.file_name() == *PACKAGE
}

#[tracing::instrument(err)]
pub async fn fetch_crds_for<'args, 'generator>(
    family: &'args str,
    version: &'args str,
) -> anyhow::Result<crate::ProviderFamilyCRDS<'generator>>
where
    'args: 'generator,
{
    tracing::debug!("fetching provider family source archive");

    let url = format!(
        "https://github.com/crossplane-contrib/provider-upjet-{family}/archive/refs/tags/v{version}.zip"
    );

    let mut src_archive = zip::ZipArchive::new(std::io::Cursor::new(
        reqwest::get(&url)
            .await
            .context("failed to fetch source archive")?
            .bytes()
            .await
            .context("failed to read response body as bytes")?,
    ))
    .context("unable to read fetched data as zip archive")?;

    let mut crds = crate::ProviderFamilyCRDS::new(family, version);

    for filepath in src_archive
        .file_names()
        .filter(is_crd_filepath)
        .map(str::to_string)
        .collect::<Vec<_>>()
    {
        let Ok(file) = src_archive.by_name(&filepath) else {
            tracing::warn!(%filepath, "unable to extract CRD file from archive");
            continue;
        };

        let Some(crd_name) = <str as AsRef<Path>>::as_ref(&filepath)
            .file_stem()
            .and_then(OsStr::to_str)
        else {
            tracing::warn!(
                %filepath,
                "unable to extract CRD file name from in-archive filepath"
            );
            continue;
        };

        let Some((provider, _)) = crd_name.split_once(".") else {
            tracing::warn!(
                %crd_name,
                "unable to extract provider name from CRD name"
            );
            continue;
        };

        let Some((_, resource)) = crd_name.split_once("_") else {
            tracing::warn!(
                %crd_name,
                "unable to extract resource name from CRD name"
            );
            continue;
        };

        let deserializer = serde_yaml::Deserializer::from_reader(file);

        let crd = match serde_path_to_error::deserialize(deserializer) {
            Ok(crd) => crd,
            Err(error) => {
                tracing::warn!(%error, "unable to deserialize CRD");
                continue;
            }
        };

        crds.entry(provider.into())
            .or_default()
            .insert(resource.into(), crd);
    }

    Ok(crds)
}
