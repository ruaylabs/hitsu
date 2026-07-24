use keepass::db::{CustomDataItem, CustomDataValue};

pub(crate) fn set_field(
    entry: &mut keepass::db::Entry,
    key: &str,
    value: Option<&str>,
    protected: bool,
) {
    match value.filter(|value| !value.is_empty()) {
        Some(value) if protected => entry.set_protected(key, value),
        Some(value) => entry.set_unprotected(key, value),
        None => {
            entry.fields.remove(key);
        }
    }
}

pub(crate) fn set_custom_data(entry: &mut keepass::db::Entry, key: &str, value: Option<&str>) {
    match value {
        Some(value) => {
            // Use Binary so keepass base64-encodes the value on serialization.
            // CustomDataValue::String is unreliable because its XML deserializer
            // tries base64 first, accidentally decoding values such as "note".
            entry.custom_data.insert(
                key.to_string(),
                CustomDataItem {
                    value: Some(CustomDataValue::Binary(value.as_bytes().to_vec())),
                    last_modification_time: None,
                },
            );
        }
        None => {
            entry.custom_data.remove(key);
        }
    }
}
