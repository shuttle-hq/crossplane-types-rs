use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use askama::Template;
use crossplane_types_generator::{cli, templates, utils};
use itertools::Itertools;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

static PROVIDER_CRATES_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    <str as AsRef<Path>>::as_ref(concat!(env!("CARGO_MANIFEST_DIR"), "/..", "/crates"))
        .canonicalize()
        .expect("failed to canonicalize on-disk path to provider crates directory")
});

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env().unwrap_or(
                concat!(
                    "info",
                    ",kopium=error",
                    ",",
                    env!("CARGO_CRATE_NAME"),
                    "=debug"
                )
                .into(),
            ),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .compact()
                .with_file(true)
                .with_line_number(true),
        )
        .init();

    let args = cli::CliArgs::parse();

    let provider_families = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        futures_util::future::join_all(
            args.provider_families()
                .iter()
                .map(|(family, version)| utils::fetch_crds_for(family, version)),
        ),
    )
    .await?
    .into_iter()
    .flatten()
    .collect_vec();

    if provider_families.is_empty() {
        anyhow::bail!("failed to fetch CRDs for any of the specified families/versions")
    }

    let workspace = templates::WorkspaceConfig::default();

    let generator = kopium::KopiumTypeGenerator::builder()
        .docs(true)
        // .builders(true)  // seems to cause extremely slow compilation 🤔
        .builders(false)
        .map_type("HashMap")
        .schema("derived")
        .smart_derive_elision(true)
        .build();

    let mut provider_crates = Vec::<templates::ProviderCrate<'_>>::new();

    for (family, version, crds) in provider_families
        .iter()
        .map(|fetched| (fetched.family, fetched.version, &fetched.crds))
        .sorted_by_key(|(family, _, _)| *family)
    {
        for (provider, resources) in crds.iter().sorted_by_key(|(key, _)| *key) {
            let feature_name = format!("{family}-{provider}");
            let crate_name = format!("crossplane-types-upbound-{feature_name}");

            let crate_path = PROVIDER_CRATES_DIR.join(&crate_name);

            let mut template = templates::ProviderCrate {
                workspace,
                crate_name,
                crate_path,
                feature_name,
                src_version: version,
                provider_name: provider,
                provider_family: Some(family),
                managed_resources: Default::default(),
            };

            let (src_dir, manifest_path) = (
                template.crate_path.join("src"),
                template.crate_path.join("Cargo.toml"),
            );

            let (lib_file, generated_dir, generated_mod_file) = (
                src_dir.join("lib.rs"),
                src_dir.join("generated"),
                src_dir.join("generated.rs"),
            );

            if args.clean
                && let Some(crate_dir) = src_dir.parent()
            {
                match std::fs::remove_dir_all(crate_dir) {
                    Ok(_) => {}
                    Err(error) => {
                        if error.kind() != std::io::ErrorKind::NotFound {
                            tracing::error!(
                                ?error,
                                %family,
                                %version,
                                %provider,
                                "failed to clean existing provider crate",
                            );

                            continue;
                        };
                    }
                }
            }

            if let Err(error) = std::fs::create_dir_all(&generated_dir) {
                tracing::error!(
                    ?error,
                    %family,
                    %version,
                    %provider,
                    "failed to create source directory for provider crate",
                );
                continue;
            };

            if !lib_file.exists() {
                let lib_code = match template.as_generated_lib().render() {
                    Ok(rendered) => rendered,
                    Err(error) => {
                        tracing::error!(
                            ?error,
                            %family,
                            %version,
                            %provider,
                            "failed to render `lib.rs` file for provider crate",
                        );
                        continue;
                    }
                };

                if let Err(error) = std::fs::write(&lib_file, lib_code) {
                    tracing::error!(
                        ?error,
                        %family,
                        %version,
                        %provider,
                        "failed to create `lib.rs` file for provider crate",
                    );
                    continue;
                }
            }

            for (resource, crd) in resources.iter().sorted_by_key(|(key, _)| *key) {
                let filepath = generated_dir.join(resource).with_extension("rs");
                let crd_types = match generator.generate_rust_types_for(crd, None).await {
                    #[allow(clippy::let_and_return)]
                    Ok(generated) => {
                        // TODO: implement a better way to do these sorts of fixups
                        let generated = generated.replace(
                            "HashMap<String, Number>",
                            "HashMap<String, serde_json::Number>",
                        );

                        generated
                    }
                    Err(error) => {
                        tracing::error!(
                            ?error,
                            %version,
                            %resource,
                            %provider,
                            %family,
                            "failed to generate types for resource",
                        );
                        continue;
                    }
                };

                if let Err(error) = std::fs::write(&filepath, crd_types) {
                    tracing::error!(
                        ?error,
                        %version,
                        %resource,
                        %provider,
                        %family,
                        "failed to write generated types for resource to disk",
                    );
                }

                template.managed_resources.push(resource);
            }

            let generated_mod_code = match template.as_generated_mod().render() {
                Ok(rendered) => rendered,
                Err(error) => {
                    tracing::error!(
                        ?error,
                        %family,
                        %version,
                        %provider,
                        "failed to render `generated.rs` file for provider crate",
                    );
                    continue;
                }
            };

            if let Err(error) = std::fs::write(&generated_mod_file, generated_mod_code) {
                tracing::error!(
                    ?error,
                    %version,
                    %provider,
                    %family,
                    "failed to write `generated.rs` file for provider crate to disk",
                );
            }

            let manifest = match template.as_manifest().render() {
                Ok(rendered) => rendered,
                Err(error) => {
                    tracing::error!(
                        ?error,
                        %family,
                        %version,
                        %provider,
                        "unable to render `Cargo.toml` manifest for provider crate",
                    );
                    continue;
                }
            };

            if let Err(error) = std::fs::write(&manifest_path, manifest) {
                tracing::error!(
                    ?error,
                    %version,
                    %provider,
                    %family,
                    "failed to write `Cargo.toml` manifest to disk for provider crate",
                );
                continue;
            };

            template.crate_path = template
                .crate_path
                .strip_prefix(PROVIDER_CRATES_DIR.as_path())
                .expect("unable to strip path prefix") // this *should* be infallible
                .to_path_buf();

            tracing::info!(
                %family,
                %version,
                %provider,
                "generated crate for provider: {}",
                template.crate_path.display(),
            );

            provider_crates.push(template);
        }
    }

    tracing::debug!("generated {} provider crates", provider_crates.len());

    Ok(())
}
