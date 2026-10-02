//! Бюджет размера файла исходника (работа w-file-size-gate, решение 40).
//!
//! Ратчет миграции: известный долг назван и заморожен в [`BASELINE`],
//! блокируется только его рост. Файл сверх бюджета без записи — отказ; файл из
//! записи, вернувшийся в бюджет, требует убрать запись — baseline только
//! уменьшается, и только отдельным решением, видным в этом списке.

use std::fs;
use std::path::{Path, PathBuf};

/// Бюджет файла исходника в строках.
const BUDGET: usize = 500;

/// Замороженный известный долг: путь от корня дерева и размер на момент
/// заморозки. Размер записи не растёт; уменьшение — убрать запись.
const BASELINE: &[(&str, usize)] = &[
    ("crates/dacc-cli/src/commit.rs", 525),
    ("crates/dacc-cli/src/gate.rs", 1458),
    ("crates/dacc-cli/src/hooks.rs", 533),
    ("crates/dacc-cli/src/message.rs", 873),
    ("crates/dacc-cli/src/work.rs", 1341),
    ("crates/dacc-cli/src/work_new.rs", 503),
    ("crates/dacc-journal/src/event.rs", 700),
    ("crates/dacc-scan/src/anchors.rs", 558),
    ("crates/dacc-scan/src/lib.rs", 1222),
];

/// Файлы исходников под бюджетом: `crates/*/src` и `crates/*/tests`.
fn sources(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(crates) = fs::read_dir(root.join("crates")) else {
        return out;
    };
    for crate_dir in crates.flatten() {
        for part in ["src", "tests"] {
            collect(&crate_dir.path().join(part), &mut out);
        }
    }
    out.sort();
    out
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

/// Размер файла в строках.
fn lines(path: &Path) -> usize {
    fs::read_to_string(path)
        .map(|text| text.lines().count())
        .unwrap_or(0)
}

/// Каждый файл исходника держит бюджет строки: сверх бюджета — только названный
/// и замороженный долг, и он не растёт.
#[test]
fn every_source_file_stays_within_the_size_budget() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut seen: Vec<String> = Vec::new();
    for path in sources(&root) {
        let relative = path
            .strip_prefix(&root)
            .unwrap_or(&path)
            .components()
            .map(|part| part.as_os_str().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join("/");
        seen.push(relative.clone());
        let size = lines(&path);
        match BASELINE.iter().find(|(name, _)| *name == relative) {
            Some((_, frozen)) => {
                assert!(
                    size <= *frozen,
                    "рост известного долга: {relative} {size} > {frozen} — разберите на модули"
                );
                assert!(
                    size > BUDGET,
                    "запись baseline устарела: {relative} вернулся в бюджет — уберите запись отдельным решением"
                );
            }
            None => assert!(
                size <= BUDGET,
                "файл сверх бюджета: {relative} {size} > {BUDGET} — разберите на модули или внесите в BASELINE отдельным решением"
            ),
        }
    }
    for (name, _) in BASELINE {
        assert!(
            seen.iter().any(|path| path == name),
            "запись baseline называет отсутствующий файл: {name} — уберите запись отдельным решением"
        );
    }
}
