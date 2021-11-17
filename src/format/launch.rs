/*!
 * De/Serialization of the configuration format of the tool
 * [polkadot-launch](https://github.com/paritytech/polkadot-launch).
 */

extern crate serde;
extern crate serde_json;

use std::io::prelude::*;
use std::collections::BTreeMap as Map;
use serde::{Serialize, Deserialize};
use serde::de::DeserializeOwned;
use serde_json::Value;

/// Polkadot-launch configuration.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub relaychain: RelayChainConfig,
    pub parachains: Vec<ParachainConfig>,
    pub simple_parachains: Vec<SimpleParachainConfig>,
    pub hrmp_channels: Vec<HrmpChannelsConfig>,
    #[serde(default)]
    pub types: Map<String, Value>,
    #[serde(default)]
    pub finalization: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayChainConfig {
    pub bin: String,
    pub chain: String,
    pub nodes: Vec<BaseNodeConfig>,
    pub genesis: Option<Value>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParachainConfig {
    pub bin: String,
    pub id: Option<String>,
    pub balance: Option<String>,
    pub chain: Option<String>,
    pub nodes: Vec<ParachainNodeConfig>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimpleParachainConfig {
    pub bin: String,
    pub id: String,
    pub port: u32,
    pub balance: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParachainNodeConfig {
    pub rpc_port: Option<u32>,
    #[serde(flatten)]
    pub base: BaseNodeConfig,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseNodeConfig {
    pub name: Option<String>,
    pub ws_port: u32,
    pub port: u32,
    pub base_path: Option<String>,
    pub flags: Option<Vec<String>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HrmpChannelsConfig {
    pub sender: u64,
    pub recipient: u64,
    pub max_capacity: u64,
    pub max_message_size: u64,
}

pub fn deserialize_str<T: AsRef<str>, D: DeserializeOwned>(s: T)
                                                           -> Result<D, impl std::error::Error> {
    serde_json::from_str(s.as_ref())
}

pub fn deserialize<R: Read>(s: R) -> Result<Config, impl std::error::Error> {
    serde_json::from_reader(s)
}

pub fn serialize_str(doc: &Config) -> Result<String, impl std::error::Error> {
    serde_json::to_string_pretty(doc)
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_LAUNCH_CFG: &str = include_str!("../../../../substrate/node/launch-config.json");

    #[test]
    fn read_example_launch_config() { deserialize_str::<_, Config>(TEST_LAUNCH_CFG).unwrap(); }
}
