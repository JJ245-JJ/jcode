#![cfg_attr(test, allow(clippy::items_after_test_module))]

pub use jcode_storage::*;

use anyhow::Result;
use serde::de::DeserializeOwned;
use std::path::Path;

pub fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    jcode_storage::read_json_with_recovery_handler(path, |event| match event {
        jcode_storage::StorageRecoveryEvent::CorruptPrimary { path, error } => {
            crate::logging::warn(&format!(
                "Corrupt JSON at {}, trying backup: {}",
                path.display(),
                error
            ));
        }
        jcode_storage::StorageRecoveryEvent::RecoveredFromBackup { backup_path } => {
            crate::logging::info(&format!("Recovered from backup: {}", backup_path.display()));
        }
    })
}

#[cfg(any(test, feature = "test-support"))]
use std::sync::{Mutex, MutexGuard, OnceLock};

#[cfg(any(test, feature = "test-support"))]
pub fn test_env_lock() -> &'static Mutex<()> {
    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    ENV_LOCK.get_or_init(|| {
        // Running `cargo test` from inside a jcode session inherits that
        // session's active-provider env, which silently re-routes provider
        // resolution in tests that never set it. Scrub it once, up front.
        for key in [
            "JCODE_NAMED_PROVIDER_PROFILE",
            "JCODE_PROVIDER_PROFILE_ACTIVE",
            "JCODE_PROVIDER_PROFILE_NAME",
            "JCODE_ACTIVE_PROVIDER",
            "JCODE_RUNTIME_PROVIDER",
            "ANTHROPIC_AUTH_TOKEN",
            "JCODE_PROVIDER_FCC_API_KEY",
        ] {
            crate::env::remove_var(key);
        }
        // An operator's `[agents] swarm_model` pin (e.g. a proxy route) would
        // make every spawn test resolve a provider the mock cannot switch to.
        // Empty clears the pin; tests that need one set it themselves.
        crate::env::set_var("JCODE_SWARM_MODEL", "");
        Mutex::new(())
    })
}

#[cfg(any(test, feature = "test-support"))]
pub fn lock_test_env() -> MutexGuard<'static, ()> {
    test_env_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests;
