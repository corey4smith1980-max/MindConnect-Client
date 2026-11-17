//! Unattended host setup: password-only approval and a fixed permanent password.
//!
//! The password is never stored in the repository. Install-time and build-time
//! values override a generated password, which is written to a local file.

use hbb_common::{config::Config, log};
use std::path::PathBuf;

pub const PASSWORD_ENV: &str = "MINDCONNECT_PERMANENT_PASSWORD";
pub const OPTION_UNATTENDED_HOST: &str = "unattended-host";
const BUILD_PASSWORD: Option<&str> = option_env!("MINDCONNECT_PERMANENT_PASSWORD");
const BUILD_UNATTENDED: Option<&str> = option_env!("MINDCONNECT_UNATTENDED");

pub fn is_unattended_build() -> bool {
    matches!(
        BUILD_UNATTENDED.map(str::trim),
        Some("1") | Some("true") | Some("yes")
    )
}

/// `(password, generated)`. An empty password means "leave the stored password".
pub fn choose_password(
    explicit: Option<&str>,
    env_value: Option<&str>,
    build_value: Option<&str>,
    already_set: bool,
    generate: impl Fn() -> String,
) -> (String, bool) {
    for candidate in [explicit, env_value, build_value] {
        if let Some(password) = candidate.map(str::trim).filter(|value| !value.is_empty()) {
            return (password.to_owned(), false);
        }
    }
    if already_set {
        return (String::new(), false);
    }
    (generate(), true)
}

pub fn password_record_path() -> PathBuf {
    #[cfg(windows)]
    {
        let base = std::env::var("ProgramData").unwrap_or_else(|_| r"C:\ProgramData".to_owned());
        return PathBuf::from(base)
            .join("BetterDesk")
            .join("unattended-password.txt");
    }
    #[cfg(not(windows))]
    {
        Config::path("unattended-password.txt")
    }
}

fn first_silent_password(args: &[String]) -> Option<String> {
    args.iter().skip(1).find_map(|arg| {
        let value = arg.strip_prefix("password=")?.trim();
        if value.is_empty() {
            None
        } else {
            Some(value.to_owned())
        }
    })
}

pub fn silent_install_password(args: &[String]) -> Option<String> {
    first_silent_password(args)
}

fn set_access_options() {
    Config::set_option("approve-mode".into(), "password".into());
    Config::set_option(
        "verification-method".into(),
        "use-permanent-password".into(),
    );
    Config::set_option(OPTION_UNATTENDED_HOST.into(), "Y".into());
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        crate::ipc::set_option("approve-mode", "password");
        crate::ipc::set_option("verification-method", "use-permanent-password");
        crate::ipc::set_option(OPTION_UNATTENDED_HOST, "Y");
    }
}

fn store_password(password: &str) -> bool {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let via_service = publish_to_running_service(password);
        let via_file = Config::set_permanent_password(password);
        return via_service || via_file;
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        Config::set_permanent_password(password)
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn publish_to_running_service(password: &str) -> bool {
    for _ in 0..20 {
        set_access_options();
        if crate::ipc::set_permanent_password(password.to_owned()).is_ok() {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    false
}

fn write_password_record(password: &str, generated: bool) -> std::io::Result<PathBuf> {
    let path = password_record_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let source = if generated {
        "generated"
    } else {
        "install-or-build"
    };
    let body = format!(
        "MindConnect unattended access password\nsource: {source}\npassword: {password}\n"
    );
    std::fs::write(&path, body)?;
    #[cfg(windows)]
    {
        let _ = std::process::Command::new("icacls")
            .arg(&path)
            .arg("/inheritance:r")
            .arg("/grant:r")
            .arg("*S-1-5-32-544:(R,W)")
            .status();
    }
    Ok(path)
}

/// Enables password-only approval and stores the permanent password.
///
/// Returns the path of the password file when one was written.
pub fn apply_unattended_host(explicit: Option<&str>) -> Option<PathBuf> {
    let env_value = std::env::var(PASSWORD_ENV).ok();
    let already_set = Config::has_permanent_password();
    let (password, generated) = choose_password(
        explicit,
        env_value.as_deref(),
        BUILD_PASSWORD,
        already_set,
        || Config::get_auto_password(12),
    );
    set_access_options();
    if password.is_empty() {
        return None;
    }
    let _stored = store_password(&password);
    match write_password_record(&password, generated) {
        Ok(path) => {
            log::info!(
                "Unattended password recorded at {} (not printed)",
                path.display()
            );
            Some(path)
        }
        Err(err) => {
            log::error!("Failed to record unattended password: {err}");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_password_wins_and_empty_generates_once() {
        let (password, generated) = choose_password(
            Some(" from-install "),
            Some("from-env"),
            Some("from-build"),
            false,
            || "generated".to_owned(),
        );
        assert_eq!(password, "from-install");
        assert!(!generated);

        let (password, generated) = choose_password(None, None, None, false, || "generated".into());
        assert_eq!(password, "generated");
        assert!(generated);

        let (password, generated) =
            choose_password(None, Some("  "), None, true, || "generated".into());
        assert!(password.is_empty());
        assert!(!generated);
    }

    #[test]
    fn silent_install_reads_password_argument() {
        let args = vec![
            "--silent-install".to_owned(),
            "printer=0".to_owned(),
            "password=s3cret".to_owned(),
        ];
        assert_eq!(silent_install_password(&args).as_deref(), Some("s3cret"));
        assert!(silent_install_password(&["--silent-install".into()]).is_none());
    }
}
