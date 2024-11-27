fn main() {
    let version = std::env::var("LEV_VERSION").unwrap_or_else(|_| "0.2.0".to_string());
    println!("cargo:rustc-env=APP_VERSION={}", version);
}
