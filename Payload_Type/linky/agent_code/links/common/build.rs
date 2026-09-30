use std::env;

fn main() {
    // Configurable User-Agent (build parameter; empty = built-in obfuscated default).
    // The builder passes USER_AGENT via env; each implant crate's build.rs forwards it too,
    // but link-common reads it here so build_client() can pick it up via option_env!().
    let user_agent = env::var("USER_AGENT").unwrap_or_default();
    println!("cargo:rustc-env=USER_AGENT={}", user_agent);
    println!("cargo:rerun-if-env-changed=USER_AGENT");
}
