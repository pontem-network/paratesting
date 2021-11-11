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
	relaychain: RelayChainConfig,
	parachains: Vec<ParachainConfig>,
	simple_parachains: Vec<SimpleParachainConfig>,
	hrmp_channels: Vec<HrmpChannelsConfig>,
	#[serde(default)]
	types: Map<String, Value>,
	#[serde(default)]
	finalization: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayChainConfig {
	bin: String,
	chain: String,
	nodes: Vec<BaseNodeConfig>,
	genesis: Option<Value>,
}


#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParachainConfig {
	bin: String,
	id: Option<String>,
	balance: Option<String>,
	chain: Option<String>,
	nodes: Vec<ParachainNodeConfig>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimpleParachainConfig {
	bin: String,
	id: String,
	port: String,
	balance: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParachainNodeConfig {
	rpc_port: Option<u32>,
	#[serde(flatten)]
	base: BaseNodeConfig,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseNodeConfig {
	name: Option<String>,
	ws_port: u32,
	port: u32,
	base_path: Option<String>,
	flags: Option<Vec<String>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HrmpChannelsConfig {
	sender: u64,
	recipient: u64,
	max_capacity: u64,
	max_message_size: u64,
}


pub fn deserialize_str<T: AsRef<str>, D: DeserializeOwned>(s: T) -> Result<D, impl std::error::Error> {
	serde_json::from_str(s.as_ref())
}

pub fn deserialize<R: Read>(s: R) -> Result<Config, impl std::error::Error> { serde_json::from_reader(s) }

pub fn serialize_str(doc: &Config) -> Result<String, impl std::error::Error> { serde_json::to_string_pretty(doc) }


#[cfg(test)]
mod tests {
	use super::*;

	static TEST_LAUNCH_CFG: &str = include_str!("../../../../substrate/node/launch-config.json");

	#[test]
	fn read_example_launch_config() { deserialize_str::<_, Config>(TEST_LAUNCH_CFG).unwrap(); }
}
