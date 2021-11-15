extern crate serde;
extern crate serde_yaml;
extern crate yaml_merge_keys;

use std::time::Duration;
use std::path::PathBuf;
use std::io::prelude::*;
use std::collections::BTreeMap as Map;
use serde::{Serialize, Deserialize};
use serde::de::DeserializeOwned;

use yaml_merge_keys::merge_keys_serde;
use serde_yaml::Value;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct TestSuitCfg {
    /// Human-readable name of the test-suit.
    pub name: Option<String>,
    /// Human-readable description.
    pub description: Option<String>,

    /// Configuration for preparation state when we running scripts and/or nodes
    /// and waiting & checking for success or failure.
    pub setup: SetupCfg,

    // TODO: doc
    pub tests: Vec<Test>,

    /// Logging configuration
    #[serde(default)]
    logging: LoggingCfg,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SetupCfg {
    #[serde(alias = "launch")]
    PolkaLaunch {
        #[serde(flatten)]
        cfg: PolkaLaunchCfg,
        #[serde(flatten)]
        conditions: ConditionsCfg,
    },
    Process {
        #[serde(flatten)]
        cfg: ProcessRunCfg<String>,
        #[serde(flatten)]
        conditions: ConditionsCfg,
        connect: ConnectCfg,
    },

    /// Just connect to already executed nodes.
    /// Not needed since using polka-launch.
    Connect {
        #[serde(flatten)]
        cfg: ConnectCfg,
    },
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct ConnectCfg {
    pub nodes: Vec<NodeCfg>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct NodeCfg {
    pub name: String,
    pub port: u32,
    pub runtime: SupportedRuntime,
    pub log_file: Option<PathBuf>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum SupportedRuntime {
    /// Generic polkadot runtime
    /// TODO: describe version here
    #[serde(alias = "polka")]
    Polkadot,

    /// TODO: describe version here
    Pontem,

    /// TODO: describe version here
    Rococo,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct PolkaLaunchCfg {
    #[serde(flatten)]
    pub inner: ProcessRunCfg<Option<String>, Option<bool>>,
    pub cfg: PathBuf,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct ProcessRunCfg<Cmd = String, KeepAlive = bool> {
    pub pwd: Option<PathBuf>,
    pub cmd: Cmd,
    // TODO: shell
    /// Keep this process alive after success/failure conditions are come.
    pub keep_alive: KeepAlive,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ConditionsCfg {
    pub success: Option<Conditions>,
    pub failure: Option<Conditions>,
    /* TODO:
    /// Check and compare conditions when reqirements are met.
    /// For example, when state satisfy the reqirements.
    reqirements or when
    #[serde(default)]
    when: Map<String, Value>,
    */
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum Conditions {
    Result(Map<String, Value>),
    Events(Map<String, Value>),
    /// Check existance of event
    Event(Option<String>),
    Stdout(String),
    Stderr(String),

    /// Check and compare conditions for every stream/state update
    // #[serde(alias = "when")]
    Wait {
        #[serde(flatten)]
        conditions: Box<Conditions>,
        #[serde(flatten)]
        limit: WaitLimit,
    },
    /// Check and compare conditions when reqirements are met.
    /// For example, when state satisfy the reqirements.
    When {
        #[serde(flatten)]
        conditions: Box<Conditions>,
        #[serde(flatten)]
        reqirements: Map<String, Value>,
    },
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum WaitLimit {
    // TODO:
    #[serde(alias = "s")]
    Secs(u64),
    #[serde(alias = "ms")]
    Millis(u64),
}

impl TryInto<Duration> for WaitLimit {
    type Error = &'static str;

    fn try_into(self) -> Result<Duration, Self::Error> {
        match self {
            WaitLimit::Secs(v) => Ok(Duration::from_secs(v)),
            WaitLimit::Millis(v) => Ok(Duration::from_millis(v)),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Test {
    pub name: String,
    pub steps: Vec<Step>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Step {
    pub name: String,

    pub nodes: Vec<String>,

    #[serde(flatten)]
    pub action: Action,

    #[serde(flatten)]
    pub conditions: ConditionsCfg,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Action {
    Call {
        #[serde(flatten)]
        data: CallData,
    },
    /// Raw rpc call
    Rpc { method: String, args: Vec<Value> },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CallData {
    pub method: String,
    pub args: Vec<String>,
    #[serde(default)]
    pub sudo: bool,
    // TODO: type of signer
    pub signer: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LoggingCfg {
    #[serde(alias = "no")]
    None,
    Tests,
    Debug,
}

impl Default for LoggingCfg {
    fn default() -> Self {
        LoggingCfg::Tests
    }
}

impl From<bool> for LoggingCfg {
    fn from(v: bool) -> Self {
        if v {
            LoggingCfg::Debug
        } else {
            LoggingCfg::None
        }
    }
}

pub fn deserialize_str<T: AsRef<str>, D: DeserializeOwned>(s: T)
                                                           -> Result<D, impl std::error::Error> {
    serde_yaml::from_str(s.as_ref()).and_then(|value| {
                                        merge_keys_serde(value).map_err(serde::de::Error::custom)
                                    })
                                    .and_then(serde_yaml::from_value)
}

pub fn deserialize<R: Read>(s: R) -> Result<TestSuitCfg, impl std::error::Error> {
    serde_yaml::from_reader(s).and_then(|value| {
                                  merge_keys_serde(value).map_err(serde::de::Error::custom)
                              })
                              .and_then(serde_yaml::from_value)
}

pub fn serialize_str(doc: &TestSuitCfg) -> Result<String, impl std::error::Error> {
    serde_yaml::to_string(doc)
}

// #[cfg(test)]
// mod tests {
// 	use super::*;

// 	static TEST_SUIT: &str = include_str!("../../examples/cases/case-a.yaml");

// 	#[test]
// 	fn read_example_case() { deserialize_str::<_, TestSuitCfg>(TEST_SUIT).unwrap(); }
// }
