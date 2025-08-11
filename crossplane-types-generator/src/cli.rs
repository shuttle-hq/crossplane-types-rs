use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

const AWS_VERSION: &str = "1.21.1";
const GCP_VERSION: &str = "1.14.0";

fn parse_provider_family_version(
    arg: &str,
) -> Result<ProviderFamilyVersion, Box<dyn 'static + Send + Sync + std::error::Error>> {
    let (key, value) = arg
        .split_once("=")
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{arg}`"))?;

    Ok((key.into(), value.into()).into())
}

#[derive(Clone, Debug)]
struct ProviderFamilyVersion((String, String));

impl Deref for ProviderFamilyVersion {
    type Target = (String, String);
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ProviderFamilyVersion {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<(String, String)> for ProviderFamilyVersion {
    fn from(platforms: (String, String)) -> Self {
        Self(platforms)
    }
}

impl<'args, 'mapped> From<&'args ProviderFamilyVersion> for (&'mapped str, &'mapped str)
where
    'args: 'mapped,
{
    fn from(value: &'args ProviderFamilyVersion) -> Self {
        (value.0.0.as_str(), value.0.1.as_str())
    }
}

impl std::fmt::Display for ProviderFamilyVersion {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}={}", &self.0.0, &self.0.1)
    }
}

#[derive(Clone, Debug, clap::Parser)]
pub struct CliArgs {
    /// Remove any existing crate files before generating new ones
    #[arg(long, default_value_t = false)]
    pub clean: bool,

    /// A mapping of platform names to target package versions
    ///
    /// E.x.:
    /// ```shell
    /// --platform 'aws=1.21.1'
    /// --platform 'gcp=1.14.0'
    /// ```
    #[arg(
        long = "platform",
        action = clap::ArgAction::Append,
        value_parser = parse_provider_family_version,
        default_values_t = [
            ProviderFamilyVersion(("aws".into(), AWS_VERSION.into())),
            ProviderFamilyVersion(("gcp".into(), GCP_VERSION.into())),
        ],
    )]
    provider_families: Vec<ProviderFamilyVersion>,
}

impl CliArgs {
    pub fn parse() -> Self {
        <Self as clap::Parser>::parse()
    }
    pub fn provider_families(&self) -> HashMap<&str, &str> {
        HashMap::from_iter(self.provider_families.iter().map(Into::into))
    }
}
