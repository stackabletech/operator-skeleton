pub mod commands;
pub mod error;

use crate::commands::{Restart, Start, Stop};

use k8s_openapi::apimachinery::pkg::apis::meta::v1::Condition;
use k8s_openapi::schemars::_serde_json::Value;
use kube::api::ApiResource;
use kube::CustomResource;
use kube::CustomResourceExt;
use schemars::JsonSchema;
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_json::json;
use stackable_operator::command::{CommandRef, HasCommands, HasRoleRestartOrder};
use stackable_operator::controller::HasOwned;
use stackable_operator::crd::HasApplication;
use stackable_operator::identity::PodToNodeMapping;
use stackable_operator::product_config_utils::{ConfigError, Configuration};
use stackable_operator::role_utils::Role;
use stackable_operator::status::{
    ClusterExecutionStatus, Conditions, HasClusterExecutionStatus, HasCurrentCommand, Status,
    Versioned,
};
use stackable_operator::versioning::{ProductVersion, Versioning, VersioningState};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use strum_macros::Display;
use strum_macros::EnumIter;

pub const APP_NAME: &str = "productname";
pub const MANAGED_BY: &str = "productname-operator";

pub const CONFIG_MAP_TYPE_DATA: &str = "data";
pub const CONFIG_MAP_TYPE_ID: &str = "id";

#[derive(Clone, CustomResource, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[kube(
    group = "productname.stackable.tech",
    version = "v1alpha1",
    kind = "ProductnameCluster",
    plural = "productnameclusters",
    shortname = "productname",
    namespaced
)]
#[kube(status = "ProductnameClusterStatus")]
pub struct ProductnameClusterSpec {
    pub version: ProductnameVersion,
    pub servers: Role<ProductnameConfig>,
}

#[derive(
    Clone, Debug, Deserialize, Display, EnumIter, Eq, Hash, JsonSchema, PartialEq, Serialize,
)]
pub enum ProductnameRole {
    #[strum(serialize = "coordinator")]
    Coordinator,
    #[strum(serialize = "worker")]
    Worker,
}

impl Status<ProductnameClusterStatus> for ProductnameCluster {
    fn status(&self) -> &Option<ProductnameClusterStatus> {
        &self.status
    }
    fn status_mut(&mut self) -> &mut Option<ProductnameClusterStatus> {
        &mut self.status
    }
}

impl HasRoleRestartOrder for ProductnameCluster {
    fn get_role_restart_order() -> Vec<String> {
        vec![ProductnameRole::Coordinator.to_string()]
    }
}

impl HasCommands for ProductnameCluster {
    fn get_command_types() -> Vec<ApiResource> {
        vec![
            Start::api_resource(),
            Stop::api_resource(),
            Restart::api_resource(),
        ]
    }
}

impl HasOwned for ProductnameCluster {
    fn owned_objects() -> Vec<&'static str> {
        vec![Restart::crd_name(), Start::crd_name(), Stop::crd_name()]
    }
}

impl HasApplication for ProductnameCluster {
    fn get_application_name() -> &'static str {
        APP_NAME
    }
}

impl HasClusterExecutionStatus for ProductnameCluster {
    fn cluster_execution_status(&self) -> Option<ClusterExecutionStatus> {
        self.status
            .as_ref()
            .and_then(|status| status.cluster_execution_status.clone())
    }

    fn cluster_execution_status_patch(&self, execution_status: &ClusterExecutionStatus) -> Value {
        json!({ "clusterExecutionStatus": execution_status })
    }
}

// TODO: These all should be "Property" Enums that can be either simple or complex where complex allows forcing/ignoring errors and/or warnings
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductnameConfig {
}

impl Configuration for ProductnameConfig {
    type Configurable = ProductnameCluster;

    fn compute_env(
        &self,
        _resource: &Self::Configurable,
        _role_name: &str,
    ) -> Result<BTreeMap<String, Option<String>>, ConfigError> {
        let mut result = BTreeMap::new();

        // TODO: Readd if we want jmx metrics gathered
        //if let Some(metrics_port) = self.metrics_port {
        //    result.insert(METRICS_PORT.to_string(), Some(metrics_port.to_string()));
        // }
        Ok(result)
    }

    fn compute_cli(
        &self,
        _resource: &Self::Configurable,
        _role_name: &str,
    ) -> Result<BTreeMap<String, Option<String>>, ConfigError> {
        Ok(BTreeMap::new())
    }

    fn compute_files(
        &self,
        _resource: &Self::Configurable,
        _role_name: &str,
        _file: &str,
    ) -> Result<BTreeMap<String, Option<String>>, ConfigError> {
        let mut result = BTreeMap::new();

        // TODO: Insert configs here

        Ok(result)
    }
}

#[allow(non_camel_case_types)]
#[derive(
    Clone,
    Debug,
    Deserialize,
    Eq,
    JsonSchema,
    PartialEq,
    Serialize,
    strum_macros::Display,
    strum_macros::EnumString,
)]
pub enum ProductnameVersion {
    #[serde(rename = "362")]
    #[strum(serialize = "362")]
    v362,

    #[serde(rename = "361")]
    #[strum(serialize = "361")]
    v361,

    #[serde(rename = "360")]
    #[strum(serialize = "360")]
    v360,
}

impl ProductnameVersion {
    pub fn package_name(&self) -> String {
        format!("productname-server-{}", self.to_string())
    }
}

impl Versioning for ProductnameVersion {
    fn versioning_state(&self, other: &Self) -> VersioningState {
        let from_version = match Version::parse(&self.to_string()) {
            Ok(v) => v,
            Err(e) => {
                return VersioningState::Invalid(format!(
                    "Could not parse [{}] to SemVer: {}",
                    self.to_string(),
                    e.to_string()
                ))
            }
        };

        let to_version = match Version::parse(&other.to_string()) {
            Ok(v) => v,
            Err(e) => {
                return VersioningState::Invalid(format!(
                    "Could not parse [{}] to SemVer: {}",
                    other.to_string(),
                    e.to_string()
                ))
            }
        };

        match to_version.cmp(&from_version) {
            Ordering::Greater => VersioningState::ValidUpgrade,
            Ordering::Less => VersioningState::ValidDowngrade,
            Ordering::Equal => VersioningState::NoOp,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductnameClusterStatus {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conditions: Vec<Condition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<ProductVersion<ProductnameVersion>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<PodToNodeMapping>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_command: Option<CommandRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster_execution_status: Option<ClusterExecutionStatus>,
}

impl Versioned<ProductnameVersion> for ProductnameClusterStatus {
    fn version(&self) -> &Option<ProductVersion<ProductnameVersion>> {
        &self.version
    }
    fn version_mut(&mut self) -> &mut Option<ProductVersion<ProductnameVersion>> {
        &mut self.version
    }
}

impl Conditions for ProductnameClusterStatus {
    fn conditions(&self) -> &[Condition] {
        self.conditions.as_slice()
    }
    fn conditions_mut(&mut self) -> &mut Vec<Condition> {
        &mut self.conditions
    }
}

impl HasCurrentCommand for ProductnameClusterStatus {
    fn current_command(&self) -> Option<CommandRef> {
        self.current_command.clone()
    }
    fn set_current_command(&mut self, command: CommandRef) {
        self.current_command = Some(command);
    }
    fn clear_current_command(&mut self) {
        self.current_command = None
    }
    fn tracking_location() -> &'static str {
        "/status/currentCommand"
    }
}

#[cfg(test)]
mod tests {
    use crate::ProductnameVersion;
    use stackable_operator::versioning::{Versioning, VersioningState};
    use std::str::FromStr;

    #[test]
    fn test_productname_version_versioning() {
        assert_eq!(
            ProductnameVersion::v3_4_14.versioning_state(&ProductnameVersion::v3_5_8),
            VersioningState::ValidUpgrade
        );
        assert_eq!(
            ProductnameVersion::v3_5_8.versioning_state(&ProductnameVersion::v3_4_14),
            VersioningState::ValidDowngrade
        );
        assert_eq!(
            ProductnameVersion::v3_4_14.versioning_state(&ProductnameVersion::v3_4_14),
            VersioningState::NoOp
        );
    }

    #[test]


    #[test]
    fn test_version_conversion() {
        // TODO: Adapt to correct product version
        // ProductnameVersion::from_str("3.4.14").unwrap();

    }

    #[test]
    fn test_package_name() {
        // TODO: Adapot to correct package names
        assert_eq!(
            ProductnameVersion::v360.package_name(),
            format!("productname-{}", ProductnameVersion::v360.to_string())
        );
        assert_eq!(
            ProductnameVersion::v360.package_name(),
            format!(
                "productname-server-{}",
                ProductnameVersion::v360.to_string()
            )
        );
    }
}
