//! Бюджет размера файла исходника (работа w-file-size-gate, решение 40).
//!
//! Ратчет миграции: известный долг назван и заморожен в [`BASELINE`] в точном
//! размере (работа w-ratchet-tight). Файл сверх бюджета без записи — отказ;
//! изменившийся размер долга — отказ с требованием обновить заморозку явным
//! решением: уменьшение фиксируется явно, возврат к замороженному размеру
//! невозможен, и запись файла, вернувшегося в бюджет, убирается отдельным
//! решением.

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
    ("crates/dacc-cli/src/work.rs", 1360),
    ("crates/dacc-cli/src/work_new.rs", 503),
    ("crates/dacc-cli/tests/work_commands.rs", 516),
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

/// Правило ратчета: файл из baseline держит точную заморозку — изменившийся
/// размер требует явного обновления записи, и только файл вне записи обязан
/// укладываться в бюджет.
fn within_budget(relative: &str, size: usize) -> Result<(), String> {
    match BASELINE.iter().find(|(name, _)| *name == relative) {
        Some((_, frozen)) if size == *frozen => {
            if size > BUDGET {
                Ok(())
            } else {
                Err(format!(
                    "запись baseline устарела: {relative} вернулся в бюджет — уберите запись отдельным решением"
                ))
            }
        }
        Some((_, frozen)) => Err(format!(
            "размер долга изменился молча: {relative} {size} != {frozen} — обновите заморозку явным решением"
        )),
        None if size <= BUDGET => Ok(()),
        None => Err(format!(
            "файл сверх бюджета: {relative} {size} > {BUDGET} — разберите на модули или внесите в BASELINE отдельным решением"
        )),
    }
}

/// Люфт заморозки закрыт (работа w-ratchet-tight): уменьшение долга требует
/// явного обновления записи, и возврат к замороженному размеру после
/// уменьшения невозможен.
#[test]
fn baseline_slack_is_a_violation() {
    let name = "crates/dacc-cli/src/work.rs";
    let frozen = BASELINE
        .iter()
        .find(|(entry, _)| *entry == name)
        .map(|(_, size)| *size)
        .expect("work.rs назван в baseline");
    assert!(within_budget(name, frozen).is_ok(), "заморозка держится");
    assert!(
        within_budget(name, frozen - 1).is_err(),
        "уменьшение без обновления заморозки — отказ"
    );
    assert!(
        within_budget(name, frozen + 1).is_err(),
        "рост сверх заморозки — отказ"
    );
}

/// Каждый файл исходника держит бюджет строки: сверх бюджета — только названный
/// и замороженный в точном размере долг.
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
        if let Err(problem) = within_budget(&relative, size) {
            panic!("{problem}");
        }
    }
    for (name, _) in BASELINE {
        assert!(
            seen.iter().any(|path| path == name),
            "запись baseline называет отсутствующий файл: {name} — уберите запись отдельным решением"
        );
    }
}
