//! Скан документов DACC и порождение модулей констант — одним вызовом
//! точки входа `dacc_scan::emit_registry` (решение 27). Пишет только в
//! `OUT_DIR`.

use std::{env, path::PathBuf};

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let sources = manifest.parent().unwrap().join("crates/dacc-work/src");
    if let Err(error) = dacc_scan::emit_registry(&manifest, &[sources.as_path()], &out) {
        println!("cargo::error=dacc: {error}");
        std::process::exit(1);
    }
    // Разметка живёт и в тестах инструмента: инвариант требует якорь на
    // исполняемый тест, а тесты ответов гоняют бинарь из сценариев.
    let tests = manifest.parent().unwrap().join("crates/dacc-cli/tests");
    if let Err(error) = dacc_scan::emit_anchors(&[sources.as_path(), tests.as_path()], &out) {
        println!("cargo::error=dacc: {error}");
        std::process::exit(1);
    }
}
