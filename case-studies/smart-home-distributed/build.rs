fn main() {
    println!("cargo:rerun-if-env-changed=CASE_STUDY_SSID");
    println!("cargo:rerun-if-env-changed=CASE_STUDY_AP_PASSWORD");

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "none" {
        // make sure linkall.x is the last linker script (otherwise might cause problems with flip-link)
        println!("cargo:rustc-link-arg=-Tlinkall.x");
    }
}
