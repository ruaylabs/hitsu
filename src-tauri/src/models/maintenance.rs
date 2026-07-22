use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EmptyRecycleBinResult {
    pub deleted_entries: usize,
}
