use std::collections::HashMap;

const AWS_VERSION: &str = "1.21.1";
const GCP_VERSION: &str = "1.14.0";

fn parse_provider_family_version(arg: &str) -> Result<(String, String), String> {
    let (key, value) = arg
        .split_once("=")
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{arg}`"))?;

    let value = value
        .strip_prefix('"')
        .unwrap_or(value)
        .strip_prefix("'")
        .unwrap_or(value)
        .strip_suffix('"')
        .unwrap_or(value)
        .strip_suffix("'")
        .unwrap_or(value);

    Ok((key.into(), value.into()))
}

/// automatic rust type generator for Crossplane CRDs
#[derive(Clone, Debug, argh::FromArgs)]
pub struct CliArgs {
    #[argh(
        switch,
        short = 'c',
        long = "clean",
        description = "remove any existing generated crates before generating new ones"
    )]
    pub clean: bool,

    #[argh(
        option,
        long = "provider",
        from_str_fn(parse_provider_family_version),
        default = r#"vec![
            ("aws".into(), AWS_VERSION.into()),
            ("gcp".into(), GCP_VERSION.into()),
        ]"#,
        description = r#"the package version to target for a given platform,
        e.x. --platform 'aws=1.21.1' --platform 'gcp=1.14.0'"#
    )]
    provider_families: Vec<(String, String)>,
}

impl CliArgs {
    #[inline(always)]
    pub fn parse() -> Self {
        argh::from_env::<Self>()
    }
    pub fn provider_families(&self) -> HashMap<&String, &String> {
        HashMap::from_iter(
            self.provider_families
                .iter()
                .map(|(key, value)| (key, value)),
        )
    }
}
