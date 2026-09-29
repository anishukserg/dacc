//! Скан реестра решений и разметки кода продукта, порождение модулей констант
//! в `OUT_DIR` (решение 27: точки входа вместо ручной оркестровки).

use std::{collections::HashSet, env, fs, path::PathBuf};

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Решения: идентификатор — slug из имени файла (решение 23).
    let decisions = match dacc_scan::scan_decisions(&manifest.join("adr")) {
        Ok(decisions) => decisions,
        Err(error) => fail(&error),
    };
    fs::write(out.join("registry.rs"), dacc_scan::emit_refs(&decisions)).unwrap();

    // Разметка кода продукта и сверка инлайн-ссылок прозы (решение 24): ссылка
    // `[id]` в прозе обязана указывать на `#[doc_anchor]` из исходников.
    let src = manifest.join("../demo-product/src");
    let anchors = match dacc_scan::scan_anchors(&[&src]) {
        Ok(anchors) => anchors,
        Err(error) => fail(&error),
    };
    let anchor_ids: HashSet<&str> = anchors.iter().map(|a| a.id.as_str()).collect();
    for decision in &decisions {
        let text = fs::read_to_string(&decision.file).unwrap();
        if let Err(error) = dacc_scan::check_inline_links(&text, &decision.file, &anchor_ids) {
            fail(&error);
        }
    }
    fs::write(
        out.join("anchors.rs"),
        dacc_scan::emit_anchor_refs(&anchors),
    )
    .unwrap();

    println!("cargo::rerun-if-changed=adr");
    println!("cargo::rerun-if-changed=../demo-product/src");
}

fn fail(error: &dacc_scan::ScanError) -> ! {
    println!("cargo::error=dacc: {error}");
    std::process::exit(1)
}
