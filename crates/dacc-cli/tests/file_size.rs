//! Бюджет размера файла исходника (работа w-file-size-gate, решение 40).
//!
//! Ратчет миграции: известный долг назван и заморожен в `BASELINE` механизмом
//! `dacc_scan::declare_baseline!` (работа w-declare-baseline). Нарушение
//! бюджета вне baseline и любое отклонение меры — отказ теста: уменьшение
//! фиксируется явным обновлением записи, рост не легализуется молча, и запись
//! файла, вернувшегося в бюджет, убирается отдельным решением.

use dacc_scan::baseline::{ratchet, Entry, Violation};
use std::fs;
use std::path::{Path, PathBuf};

/// Бюджет файла исходника в строках.
const BUDGET: usize = 500;

/// Замороженный известный долг: файл сверх бюджета и его размер на момент
/// заморозки. Запись только уменьшается — и только явным решением.
const BASELINE: &[Entry] = dacc_scan::declare_baseline!(
    "crates/dacc-cli/src/commit.rs" => 525,
    "crates/dacc-cli/src/gate.rs" => 1458,
    "crates/dacc-cli/src/hooks.rs" => 533,
    "crates/dacc-cli/src/message.rs" => 909,
    "crates/dacc-cli/src/work.rs" => 1277,
    "crates/dacc-cli/src/work_new.rs" => 503,
    "crates/dacc-cli/tests/work_commands.rs" => 634,
    "crates/dacc-journal/src/event.rs" => 700,
    "crates/dacc-scan/src/anchors.rs" => 558,
    "crates/dacc-scan/src/lib.rs" => 1223,
);

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

/// Люфт заморозки закрыт (работы w-ratchet-tight и w-declare-baseline):
/// уменьшение меры рядом с заморозкой требует явного обновления baseline, и
/// возврат к замороженному размеру после уменьшения невозможен.
#[dacc_derive::doc_anchor(id = "baseline-slack-is-a-violation")]
#[test]
fn baseline_slack_is_a_violation() {
    let name = "crates/dacc-cli/src/work.rs";
    let frozen = BASELINE
        .iter()
        .find(|entry| entry.name == name)
        .copied()
        .expect("work.rs назван в baseline");
    let only = [frozen];
    let at = |measure| {
        vec![Violation {
            name: name.to_owned(),
            measure,
        }]
    };
    assert!(
        ratchet(&at(frozen.measure), &only).is_ok(),
        "заморозка держится"
    );
    assert!(
        ratchet(&at(frozen.measure - 1), &only).is_err(),
        "уменьшение без обновления baseline — отказ"
    );
    assert!(
        ratchet(&at(frozen.measure + 1), &only).is_err(),
        "рост сверх baseline — отказ"
    );
}

/// Каждый файл исходника держит бюджет строки: нарушения бюджета сверяются с
/// замороженным baseline ратчетом — вне записи и с изменившейся мерой они не
/// проходят.
#[test]
fn every_source_file_stays_within_the_size_budget() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut violations: Vec<Violation> = Vec::new();
    for path in sources(&root) {
        let relative = path
            .strip_prefix(&root)
            .unwrap_or(&path)
            .components()
            .map(|part| part.as_os_str().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join("/");
        let size = lines(&path);
        if size > BUDGET {
            violations.push(Violation {
                name: relative,
                measure: size,
            });
        }
    }
    if let Err(problem) = ratchet(&violations, BASELINE) {
        panic!("{problem} — разберите на модули или обновите baseline отдельным решением");
    }
}
