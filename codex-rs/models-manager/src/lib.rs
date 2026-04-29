pub(crate) mod cache;
pub mod collaboration_mode_presets;
pub(crate) mod config;
pub mod manager;
pub mod model_info;
pub mod model_presets;

pub use codex_app_server_protocol::AuthMode;
pub use codex_login::AuthManager;
pub use codex_login::CodexAuth;
pub use codex_model_provider_info::ModelProviderInfo;
pub use codex_model_provider_info::WireApi;
pub use config::ModelsManagerConfig;

/// Load the bundled model catalog shipped with `codex-models-manager`.
pub fn bundled_models_response()
-> std::result::Result<codex_protocol::openai_models::ModelsResponse, serde_json::Error> {
    serde_json::from_str(include_str!("../models.json"))
}

/// Convert the client version string to a whole version string (e.g. "1.2.3-alpha.4" -> "1.2.3").
pub fn client_version_to_whole() -> String {
    const MIN_REMOTE_COMPAT_VERSION: (u64, u64, u64) = (0, 99, 0);

    let current = (
        env!("CARGO_PKG_VERSION_MAJOR").parse::<u64>().unwrap_or(0),
        env!("CARGO_PKG_VERSION_MINOR").parse::<u64>().unwrap_or(0),
        env!("CARGO_PKG_VERSION_PATCH").parse::<u64>().unwrap_or(0),
    );
    let effective = if current < MIN_REMOTE_COMPAT_VERSION {
        MIN_REMOTE_COMPAT_VERSION
    } else {
        current
    };

    format!("{}.{}.{}", effective.0, effective.1, effective.2)
}
