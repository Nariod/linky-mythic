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

    println!("cargo:rerun-if-env-changed=CALLBACK");
    println!("cargo:rerun-if-env-changed=IMPLANT_SECRET");
    println!("cargo:rerun-if-env-changed=PAYLOAD_UUID");
}
