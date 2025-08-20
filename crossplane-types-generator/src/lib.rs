use std::collections::HashMap;

pub use cargo_toml::{Manifest as CargoManifest, Value as TomlValue};
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;

pub mod cli;
pub mod templates;
pub mod utils;

pub type ProviderName = String;
pub type ResourceName = String;
pub type ProviderCRDs = HashMap<ResourceName, CustomResourceDefinition>;
pub type WorkspaceManifest = cargo_toml::Workspace<TomlValue>;

#[derive(Clone, Debug)]
pub struct ProviderFamilyCRDS<'a> {
    pub family: &'a str,
    pub version: &'a str,
    pub crds: HashMap<ProviderName, ProviderCRDs>,
}

impl<'args, 'generator> ProviderFamilyCRDS<'generator>
where
    'args: 'generator,
{
    pub fn new(family: &'args str, version: &'args str) -> Self {
        Self {
            family,
            version,
            crds: Default::default(),
        }
    }
}

impl std::ops::Deref for ProviderFamilyCRDS<'_> {
    type Target = HashMap<ProviderName, ProviderCRDs>;

    fn deref(&self) -> &Self::Target {
        &self.crds
    }
}

impl std::ops::DerefMut for ProviderFamilyCRDS<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.crds
    }
}
