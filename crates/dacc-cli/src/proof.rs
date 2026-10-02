//! Доказательство готовности (решение 15): калитка прошла на дереве, хэш
//! которого считается без каталога журнала.
//!
//! Событие журнала само меняет дерево коммита, поэтому доказательство
//! привязано не к хэшу коммита и не к хэшу дерева git, а к git-хэшу списка
//! `git ls-tree -r -z` дерева без каталога журнала. Разделитель — нулевой байт:
//! список не зависит от настройки core.quotePath, и хэш одного дерева
//! одинаков у всех.

use crate::{git, layout};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Stdio;

/// Хэш дерева без журнала: и без прежнего каталога `journal/`, и без одной
/// записи `journal.toml` (решение 43). `journal` — каталог журнала из настройки
/// того дерева, о котором идёт речь (решение 20), `tree` — любой указатель на
/// дерево: sha дерева, коммит, `HEAD`.
pub fn content_hash(root: &Path, journal: &str, tree: &str) -> Option<String> {
    let listing = git::read(root, &["ls-tree", "-r", "-z", "--full-tree", tree])?;
    let journal_dir = format!("{journal}/");
    let journal_file = format!("{journal}.toml");
    let mut kept = String::new();
    for entry in listing.split('\0').filter(|entry| !entry.is_empty()) {
        let path = entry.split_once('\t').map_or("", |(_, path)| path);
        if !path.starts_with(&journal_dir) && path != journal_file {
            kept.push_str(entry);
            kept.push('\0');
        }
    }
    hash_stdin(root, &kept)
}

/// Хэш дерева коммита, который сейчас собирается: индекса, на который
/// указывает GIT_INDEX_FILE хука.
pub fn index_hash(root: &Path, journal: &str) -> Option<String> {
    let tree = git::read(root, &["write-tree"])?;
    content_hash(root, journal, &tree)
}

/// git-хэши файлов по содержимому, без фильтров git, в порядке путей; `None` —
/// хэш файла не посчитан.
pub fn file_hashes(root: &Path, files: &[PathBuf]) -> Vec<Option<String>> {
    if files.is_empty() {
        return Vec::new();
    }
    let mut input = String::new();
    for file in files {
        input.push_str(&file.to_string_lossy());
        input.push('\n');
    }
    let output = run_with_input(
        root,
        &["hash-object", "--no-filters", "--stdin-paths"],
        &input,
    );
    let mut hashes: Vec<Option<String>> = output
        .unwrap_or_default()
        .lines()
        .map(|line| Some(line.trim().to_owned()))
        .collect();
    hashes.resize(files.len(), None);
    hashes
}

/// Путь доказательства для хэша в общем каталоге git.
///
/// Доказательство принадлежит репозиторию, а не рабочей копии, в которой
/// прошла калитка: сессии изолируются связанными копиями (решение 4), и
/// проверка одного и того же дерева не должна исчезать вместе с копией.
pub fn path(common_dir: &Path, hash: &str) -> PathBuf {
    common_dir.join(layout::PROOFS_DIR).join(hash)
}

/// Сохраняет доказательство: время, человекочитаемый вердикт и структурный
/// вердикт в JSON (решение 22). Структура прежних доказательств отсутствует.
pub fn record(
    common_dir: &Path,
    hash: &str,
    verdict: &str,
    structured: Option<&str>,
) -> io::Result<()> {
    let file = path(common_dir, hash);
    if let Some(dir) = file.parent() {
        fs::create_dir_all(dir)?;
    }
    let mut text = format!("{}\n{verdict}\n", dacc_journal::time::now());
    if let Some(structured) = structured {
        text.push_str(structured);
        text.push('\n');
    }
    fs::write(file, text)
}

/// Человекочитаемый вердикт сохранённого доказательства для хэша.
pub fn verdict(common_dir: &Path, hash: &str) -> Option<String> {
    let text = fs::read_to_string(path(common_dir, hash)).ok()?;
    text.lines().nth(1).map(str::to_owned)
}

/// RedBefore доказывает тест дерева (работа w-red-before-test): имя функции
/// совпадает точно — `fn <name>(` — и над функцией стоит атрибут, в имени
/// которого есть test (`#[test]`, `#[tokio::test]`, `#[rstest]`). Обычная
/// функция и префикс имени доказательством не считаются.
pub fn red_before_is_a_test(root: &Path, commit: &str, name: &str) -> bool {
    let hits = git::read(
        root,
        &[
            "grep",
            "-l",
            "-e",
            &format!("fn {name}("),
            commit,
            "--",
            "*.rs",
        ],
    )
    .unwrap_or_default();
    // git grep по ревизии префиксует каждый путь её именем — снимается оно.
    let prefix = format!("{commit}:");
    for hit in hits.lines().map(str::trim).filter(|hit| !hit.is_empty()) {
        let file = hit.strip_prefix(&prefix).unwrap_or(hit);
        let Some(text) = git::read(root, &["show", &format!("{commit}:{file}")]) else {
            continue;
        };
        if fn_has_test_attribute(&text, name) {
            return true;
        }
    }
    false
}

/// Функция `fn <name>(` в тексте несёт атрибут с test непосредственно над ней:
/// блок атрибутов и doc-комментарии над функцией просматриваются сверху вниз,
/// первый не-атрибут заканчивает просмотр.
fn fn_has_test_attribute(text: &str, name: &str) -> bool {
    let signature = format!("fn {name}(");
    let lines: Vec<&str> = text.lines().collect();
    for (index, line) in lines.iter().enumerate() {
        if !line.trim_start().starts_with(&signature) {
            continue;
        }
        let mut cursor = index;
        while cursor > 0 {
            cursor -= 1;
            let trimmed = lines[cursor].trim();
            if trimmed.is_empty() || trimmed.starts_with("///") {
                continue;
            }
            if !trimmed.starts_with("#[") {
                break;
            }
            if trimmed.contains("test") {
                return true;
            }
        }
    }
    false
}

fn hash_stdin(root: &Path, text: &str) -> Option<String> {
    run_with_input(root, &["hash-object", "--stdin"], text).map(|out| out.trim().to_owned())
}

/// Вывод git с данным стандартным вводом; `None` — git отказал.
fn run_with_input(root: &Path, args: &[&str], input: &str) -> Option<String> {
    let mut child = git::command(root)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    child.stdin.take()?.write_all(input.as_bytes()).ok()?;
    let output = child.wait_with_output().ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}
