//! Скан документов Slipway и порождение модулей констант — одним вызовом
//! точки входа `slipway_scan::emit_registry` (решение 27). Пишет только в
//! `OUT_DIR`.

use std::{env, path::PathBuf};

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    if let Err(error) = slipway_scan::emit_registry(&manifest, &out) {
        println!("cargo::error=slipway: {error}");
        std::process::exit(1);
    }
}
