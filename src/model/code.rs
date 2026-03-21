use serde::{Deserialize, Serialize};

use crate::PearlError;

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

impl CodeRule {
    pub(crate) fn check(&self) -> Result<(), PearlError> {
        let slot_len = self
            .default
            .iter()
            .filter(|item| !matches!(item, CodeItem::Space))
            .count();

        let mut used_by_any_cap = vec![false; slot_len];
        for cap in &self.extra.caps {
            let mut seen = vec![false; slot_len];
            for &bit in &cap.bits {
                if bit == 0 || bit > slot_len {
                    return Err(PearlError::InvalidCapBit { bit, max: slot_len });
                }
                let idx = bit - 1;
                if seen[idx] {
                    return Err(PearlError::DuplicateCapBit { bit });
                }
                if used_by_any_cap[idx] {
                    return Err(PearlError::OverlappingCapBit { bit });
                }
                seen[idx] = true;
                used_by_any_cap[idx] = true;
            }
        }
        Ok(())
    }
}
