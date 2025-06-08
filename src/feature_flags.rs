use std::env;
use std::sync::LazyLock;

pub struct FeatureFlags {
    pub is_ephemeral_db_allowed: bool,
}

pub static FEATURE_FLAGS: LazyLock<FeatureFlags> = LazyLock::new(|| {
    let is_ephemeral_db_allowed: bool = env::var("EPHEMERAL_DB_ALLOWED")
        .unwrap_or_else(|_| "false".to_string())
        .eq_ignore_ascii_case("true");

    FeatureFlags {
        is_ephemeral_db_allowed,
    }
});
