use serde::{Serialize, Deserialize, de::DeserializeOwned};


#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Config<Conditions> {
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
