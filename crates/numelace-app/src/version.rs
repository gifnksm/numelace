//! Build/version helpers for worker compatibility checks.

/// Returns a combined version string: `pkg_version (git_hash)`.
///
/// If git metadata is unavailable, the hash is reported as `unknown`.
#[must_use]
pub fn build_version() -> String {
    let pkg_version = env!("CARGO_PKG_VERSION");
    let git_hash = option_env!("VERGEN_GIT_SHA").unwrap_or("unknown");

    format!("{pkg_version} ({git_hash})")
}
