//! Скан реестра решений и разметки кода продукта, порождение модулей констант
//! в `OUT_DIR` (решение 27: точки входа вместо ручной оркестровки).

use std::{env, fs, path::PathBuf};

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Решения: идентификатор — slug из имени файла (решение 23).
    let decisions = match dacc_scan::scan_decisions(&manifest.join("adr")) {
        Ok(decisions) => decisions,
        Err(error) => fail(&error),
    };
    fs::write(out.join("registry.rs"), dacc_scan::emit_refs(&decisions)).unwrap();

    // Разметка кода продукта: точка входа (решение 27).
    if let Err(error) = dacc_scan::emit_anchors(&[&manifest.join("../demo-product/src")], &out) {
        fail(&error);
    }

    println!("cargo::rerun-if-changed=adr");
    println!("cargo::rerun-if-changed=../demo-product/src");
}

fn fail(error: &dacc_scan::ScanError) -> ! {
    println!("cargo::error=dacc: {error}");
    std::process::exit(1)
}
