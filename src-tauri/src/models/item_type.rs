use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Zeroize, ZeroizeOnDrop)]
#[serde(rename_all = "snake_case")]
pub enum ItemType {
    Login,
    Password,
    Note,
    Identity,
    Card,
    SoftwareLicense,
}

impl ItemType {
    pub fn from_db_value(s: &str) -> Self {
        match s {
            "login" => ItemType::Login,
            "password" => ItemType::Password,
            "note" => ItemType::Note,
            "identity" => ItemType::Identity,
            "card" => ItemType::Card,
            "software_license" => ItemType::SoftwareLicense,
            _ => ItemType::Login,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ItemType;

    #[test]
    fn reads_password_item_type() {
        assert_eq!(ItemType::from_db_value("password"), ItemType::Password);
    }

    #[test]
    fn reads_software_license_item_type() {
        assert_eq!(
            ItemType::from_db_value("software_license"),
            ItemType::SoftwareLicense
        );
    }
}
