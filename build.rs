#[cfg(windows)]
fn build_windows() {
    let file = "src/platform/windows.cc";
    let file2 = "src/platform/windows_delete_test_cert.cc";
    cc::Build::new().file(file).file(file2).compile("windows");
    println!("cargo:rustc-link-lib=WtsApi32");
    println!("cargo:rerun-if-changed={}", file);
    println!("cargo:rerun-if-changed={}", file2);
}

#[cfg(target_os = "macos")]
fn build_mac() {
    let file = "src/platform/macos.mm";
    let mut b = cc::Build::new();
    if let Ok(os_version::OsVersion::MacOS(v)) = os_version::detect() {
        let v = v.version;
        if v.contains("10.14") {
            b.flag("-DNO_InputMonitoringAuthStatus=1");
        }
    }
    b.flag("-std=c++17").file(file).compile("macos");
    println!("cargo:rerun-if-changed={}", file);
}

#[cfg(all(windows, feature = "inline"))]
fn build_manifest() {
    use std::io::Write;
    if std::env::var("PROFILE").unwrap() == "release" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("res/icon.ico")
            .set_language(winapi::um::winnt::MAKELANGID(
                winapi::um::winnt::LANG_ENGLISH,
                winapi::um::winnt::SUBLANG_ENGLISH_US,
            ))
            .set_manifest_file("res/manifest.xml");
        match res.compile() {
            Err(e) => {
                write!(std::io::stderr(), "{}", e).unwrap();
                std::process::exit(1);
            }
            Ok(_) => {}
        }
    }
}

fn install_android_deps() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    if target_os != "android" {
        return;
    }
    let mut target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    if target_arch == "x86_64" {
        target_arch = "x64".to_owned();
    } else if target_arch == "x86" {
        target_arch = "x86".to_owned();
    } else if target_arch == "aarch64" {
        target_arch = "arm64".to_owned();
    } else {
        target_arch = "arm".to_owned();
    }
    let target = format!("{}-android", target_arch);
    let vcpkg_root = std::env::var("VCPKG_ROOT").unwrap();
    let mut path: std::path::PathBuf = vcpkg_root.into();
    if let Ok(vcpkg_root) = std::env::var("VCPKG_INSTALLED_ROOT") {
        path = vcpkg_root.into();
    } else {
        path.push("installed");
    }
    path.push(target);
    println!(
        "cargo:rustc-link-search={}",
        path.join("lib").to_str().unwrap()
    );
    println!("cargo:rustc-link-lib=ndk_compat");
    println!("cargo:rustc-link-lib=c++");
    println!("cargo:rustc-link-lib=OpenSLES");
}

fn json_string_field(text: &str, field: &str) -> Option<String> {
    let needle = format!("\"{field}\"");
    let idx = text.find(&needle)?;
    let rest = &text[idx + needle.len()..];
    let colon = rest.find(':')?;
    let rest = rest[colon + 1..].trim_start();
    if !rest.starts_with('"') {
        return None;
    }
    let mut out = String::new();
    let mut chars = rest[1..].chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => match chars.next()? {
                '"' => out.push('"'),
                '\\' => out.push('\\'),
                '/' => out.push('/'),
                'n' => out.push('\n'),
                'r' => out.push('\r'),
                't' => out.push('\t'),
                other => out.push(other),
            },
            '"' => return Some(out),
            _ => out.push(c),
        }
    }
    None
}

fn json_escape(value: &str) -> String {
    let mut out = String::new();
    for c in value.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

fn env_or_file(file_value: String, var: &str) -> String {
    match std::env::var(var) {
        Ok(value) => {
            let value = value.trim();
            if value.is_empty() {
                file_value
            } else {
                value.to_owned()
            }
        }
        Err(_) => file_value,
    }
}

fn endpoint_host(value: &str) -> String {
    let mut host = value.trim();
    if let Some(idx) = host.find("://") {
        host = &host[idx + 3..];
    }
    host = host.split('/').next().unwrap_or(host);
    if let Some(idx) = host.rfind('@') {
        host = &host[idx + 1..];
    }
    if let Some(rest) = host.strip_prefix('[') {
        if let Some(end) = rest.find(']') {
            return rest[..end].trim_end_matches('.').to_ascii_lowercase();
        }
    }
    host.split(':')
        .next()
        .unwrap_or(host)
        .trim_end_matches('.')
        .to_ascii_lowercase()
}

fn is_public_rustdesk_host(host: &str) -> bool {
    host == "rustdesk.com" || host.ends_with(".rustdesk.com")
}

fn require_private_endpoint(label: &str, value: &str) {
    let host = endpoint_host(value);
    if host.is_empty() || is_public_rustdesk_host(&host) {
        panic!("{label} must be this deployment's server, not a public RustDesk host");
    }
}

/// Compile `res/mindconnect/server.json` into the override payload every platform links.
/// Non-empty `MINDCONNECT_*` environment variables replace the committed defaults.
fn bake_mindconnect_server() {
    use hbb_common::base64::Engine;

    const SOURCE: &str = "res/mindconnect/server.json";
    println!("cargo:rerun-if-changed={SOURCE}");
    for var in [
        "MINDCONNECT_ID_SERVER",
        "MINDCONNECT_RELAY_SERVER",
        "MINDCONNECT_API_SERVER",
        "MINDCONNECT_KEY",
    ] {
        println!("cargo:rerun-if-env-changed={var}");
    }

    let text = std::fs::read_to_string(SOURCE)
        .unwrap_or_else(|err| panic!("failed to read {SOURCE}: {err}"));
    let id_server = env_or_file(
        json_string_field(&text, "id_server")
            .unwrap_or_else(|| panic!("{SOURCE} is missing id_server")),
        "MINDCONNECT_ID_SERVER",
    );
    let relay_server = env_or_file(
        json_string_field(&text, "relay_server")
            .unwrap_or_else(|| panic!("{SOURCE} is missing relay_server")),
        "MINDCONNECT_RELAY_SERVER",
    );
    let api_server = env_or_file(
        json_string_field(&text, "api_server")
            .unwrap_or_else(|| panic!("{SOURCE} is missing api_server")),
        "MINDCONNECT_API_SERVER",
    );
    let key = env_or_file(
        json_string_field(&text, "key").unwrap_or_else(|| panic!("{SOURCE} is missing key")),
        "MINDCONNECT_KEY",
    );

    for (label, value) in [
        ("MINDCONNECT_ID_SERVER", id_server.trim()),
        ("MINDCONNECT_RELAY_SERVER", relay_server.trim()),
        ("MINDCONNECT_API_SERVER", api_server.trim()),
        ("MINDCONNECT_KEY", key.trim()),
    ] {
        if value.is_empty() {
            panic!("{label} is empty");
        }
    }
    require_private_endpoint("MINDCONNECT_ID_SERVER", id_server.trim());
    require_private_endpoint("MINDCONNECT_RELAY_SERVER", relay_server.trim());
    require_private_endpoint("MINDCONNECT_API_SERVER", api_server.trim());
    let api = api_server.trim();
    if !(api.starts_with("http://") || api.starts_with("https://")) {
        panic!("MINDCONNECT_API_SERVER must start with http:// or https://");
    }
    let key_bytes = hbb_common::base64::engine::general_purpose::STANDARD
        .decode(key.trim())
        .unwrap_or_else(|_| panic!("MINDCONNECT_KEY is not standard base64"));
    if key_bytes.len() != 32 {
        panic!("MINDCONNECT_KEY must decode to a 32-byte Ed25519 public key");
    }

    let payload = format!(
        concat!(
            "{{\n",
            "  \"override-settings\": {{\n",
            "    \"custom-rendezvous-server\": \"{}\",\n",
            "    \"relay-server\": \"{}\",\n",
            "    \"api-server\": \"{}\",\n",
            "    \"key\": \"{}\",\n",
            "    \"hide-server-settings\": \"Y\"\n",
            "  }}\n",
            "}}\n"
        ),
        json_escape(id_server.trim()),
        json_escape(relay_server.trim()),
        json_escape(api),
        json_escape(key.trim()),
    );
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_path = std::path::Path::new(&out_dir).join("mindconnect_server.json");
    std::fs::write(&out_path, payload)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", out_path.display()));
}

fn validate_embedded_support_config() {
    println!("cargo:rerun-if-env-changed=BETTERDESK_SUPPORT_CONFIG_B64");
    println!("cargo:rerun-if-env-changed=BETTERDESK_GENERIC_SUPPORT");

    let Ok(value) = std::env::var("BETTERDESK_SUPPORT_CONFIG_B64") else {
        return;
    };
    let value = value.trim();
    if value.is_empty() {
        return;
    }
    if value.len() < 128
        || value.len() % 4 != 0
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/' | b'='))
    {
        panic!("BETTERDESK_SUPPORT_CONFIG_B64 is not a valid signed base64 payload");
    }
}

fn main() {
    hbb_common::gen_version();
    bake_mindconnect_server();
    validate_embedded_support_config();
    install_android_deps();
    #[cfg(all(windows, feature = "inline"))]
    build_manifest();
    #[cfg(windows)]
    build_windows();
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    if target_os == "macos" {
        #[cfg(target_os = "macos")]
        build_mac();
        println!("cargo:rustc-link-lib=framework=ApplicationServices");
    }
    println!("cargo:rerun-if-changed=build.rs");
}
