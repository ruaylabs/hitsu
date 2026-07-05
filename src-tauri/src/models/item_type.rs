use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Zeroize, ZeroizeOnDrop)]
#[serde(rename_all = "snake_case")]
pub enum ItemType {
    Login,
    Note,
    Identity,
    Card,
}

impl ItemType {
    pub fn from_db_value(s: &str) -> Self {
        match s {
            "login" => ItemType::Login,
            "note" => ItemType::Note,
            "identity" => ItemType::Identity,
            "card" => ItemType::Card,
            _ => ItemType::Login,
        }
    }
}
