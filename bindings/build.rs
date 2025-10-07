use cbindgen::{Config, DocumentationStyle, ParseConfig};
use std::env;
use std::path::PathBuf;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let output_file = target_dir()
        .expect("could not find target dir")
        .join("bindings.hpp")
        .display()
        .to_string();

    let config = Config {
        namespace: Some(String::from("ember::__ffi")),
        parse: ParseConfig {
            parse_deps: true,
            include: Some(Vec::from(["ember".into(), "ember-core".into()])),
            ..Default::default()
        },
        no_includes: true,
        includes: Vec::from(["inttypes.h".into()]),
        include_guard: Some("EMBER_CORE_H".into()),
        documentation_style: DocumentationStyle::Doxy,
        ..Default::default()
    };

    cbindgen::generate_with_config(&crate_dir, config)
        .unwrap()
        .write_to_file(&output_file);
}

fn target_dir() -> Option<PathBuf> {
    if let Ok(target) = env::var("CARGO_TARGET_DIR") {
        Some(PathBuf::from(target))
    } else {
        let mut current = PathBuf::from(env::var("CARGO_MANIFEST_DIR").ok()?);
        while !current.join("target").exists() {
            // Go up a directory.
            current = current.join("../");

            if !current.exists() {
                // We cannot go higher up the tree.
                return None;
            }
        }
        Some(current.join("target"))
    }
}
