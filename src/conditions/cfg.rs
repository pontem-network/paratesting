use serde::{Serialize, Deserialize};


#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ConditionsCfg<Conditions> {
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
