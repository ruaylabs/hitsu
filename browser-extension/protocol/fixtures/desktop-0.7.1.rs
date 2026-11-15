use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub(super) enum BrowserRequest {
    ListLogins {
        token: String,
        origin: String,
        /// True when the request backs the inline suggestion menu rendered
        /// into page DOM. Required so the matching policy below cannot
        /// silently fall back to the broader popup matching.
        inline: bool,
    },
    GetCredentials {
        token: String,
        id: String,
        origin: String,
        inline: bool,
    },
    GetTotp {
        token: String,
        id: String,
        origin: String,
    },
}

