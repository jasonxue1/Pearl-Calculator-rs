use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum CodeItem {
    Red { count: u64 },
    Blue { count: u64 },
    Direction { count: u64 },
    Space,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CodeExtra {
    pub caps: Vec<CodeCaps>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CodeCaps {
    pub bits: Vec<usize>,
    pub cap: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CodeRule {
    pub default: Vec<CodeItem>,
    pub extra: CodeExtra,
}
