//! Слой доступа (решение 21, RFC-0003): карта реестра, место в коде и
//! директивы агентов. Ответы идут из дерева реестра и свёртки журнала, а не
//! из обхода каталогов и текстового поиска: навигация не требует find, grep
//! и ls.
//!
//! Модули — по командам (работа w-access-split); разбор аргументов, форма
//! строк JSON и XML и чтение записей реестра живут в одном экземпляре:
//! [`scan_args`], [`crate::format::string`], [`crate::record`].

mod emit;
mod form;
mod help;
mod map;
mod place;
mod registry;
mod summary;

use crate::code;
use crate::format::{self, Format};
use crate::work::{refused, usage, Context, Refusal};
use std::ffi::OsString;
use std::fs;

pub(crate) use form::error_json;

/// `cargo dacc map [--format json]` — карта реестра одной командой.
pub fn run_map(args: &[OsString]) -> u8 {
    finish_json(peek_format(args), map::run(args))
}

/// `cargo dacc where --file <path> [--format json]` — что известно о месте
/// в коде: его разметка, документы, ссылающиеся на неё, и связанные работы.
pub fn run_where(args: &[OsString]) -> u8 {
    finish_json(peek_format(args), place::run(args))
}

/// `cargo dacc emit all [--check]` — файлы-директивы агентов. Записанный файл
/// коммитится обычным ритуалом; `--check` сверяет свежесть без записи.
pub fn run_emit(args: &[OsString]) -> u8 {
    finish_json(peek_format(args), emit::run(args))
}

/// `cargo dacc ls <kind> [--status <status>] [--format json]` — перечень
/// записей реестра одного вида с их статусом или состоянием.
pub fn run_ls(args: &[OsString]) -> u8 {
    finish_json(peek_format(args), registry::ls(args))
}

/// `cargo dacc show <id> [--format json]` — запись реестра целиком.
pub fn run_show(args: &[OsString]) -> u8 {
    finish_json(peek_format(args), registry::show(args))
}

/// `cargo dacc refs <id> [--format json]` — граф записи: её ссылки и записи,
/// ссылающиеся на неё.
pub fn run_refs(args: &[OsString]) -> u8 {
    finish_json(peek_format(args), registry::refs(args))
}

/// `cargo dacc find <text> [--format json]` — поиск по идентификаторам,
/// заголовкам и текстам записей реестра.
pub fn run_find(args: &[OsString]) -> u8 {
    finish_json(peek_format(args), registry::find(args))
}

/// `cargo dacc brief [--format json]` — постоянная сводка: направления,
/// открытые срезы и открытые работы.
pub fn run_brief(args: &[OsString]) -> u8 {
    finish_json(peek_format(args), summary::brief(args))
}

/// `cargo dacc state [--format json]` — изменяющееся за день: события дня и
/// открытая работа. Потолок 2 КБ, усечение явное (RFC-0003).
pub fn run_state(args: &[OsString]) -> u8 {
    finish_json(peek_format(args), summary::state(args))
}

/// `cargo dacc help [--format json]` — машинный контракт слоя доступа:
/// каждая команда объявляет стоимость и мутирование (RFC-0003).
pub fn run_help(args: &[OsString]) -> u8 {
    finish_json(peek_format(args), help::run(args))
}

/// Формат из аргументов — для выбора формы отказа до разбора команды.
fn peek_format(args: &[OsString]) -> Format {
    let mut format = Format::Text;
    let mut words = args.iter();
    while let Some(flag) = words.next() {
        if flag.to_string_lossy() == "--format" {
            if let Some(value) = words.next() {
                if let Ok(parsed) = format::parse(&value.to_string_lossy()) {
                    format = parsed;
                }
            }
        }
    }
    format
}

/// Завершение команды слоя доступа: в json законный отказ несёт поле
/// legitimate и отличается от сбоя (RFC-0003).
fn finish_json(format: Format, outcome: Result<u8, Refusal>) -> u8 {
    match outcome {
        Ok(code) => code,
        Err(refusal) if format == Format::Json => {
            println!(
                "{{\"schema\": \"dacc-error\", \"legitimate\": true, \"code\": {}, \"reason\": {}}}",
                crate::format::string(refusal.code()),
                crate::format::string(refusal.reason())
            );
            refusal.exit_code()
        }
        Err(refusal) => crate::work::finish(Err(refusal)),
    }
}

/// Позиционный аргумент и флаги команды слоя доступа — один разбор
/// аргументов на все команды (работа w-access-split).
pub(super) struct Scan {
    pub(super) position: Option<String>,
    pub(super) format: Format,
    pub(super) status: Option<String>,
    pub(super) file: Option<String>,
    pub(super) check: bool,
}

/// Разбор аргументов команды слоя доступа: позиционный аргумент, `--format`,
/// `--status`, `--file` и `--check`. Неизвестный флаг и флаг без значения —
/// отказ с именем флага.
pub(super) fn scan_args(args: &[OsString], accepted: &str) -> Result<Scan, Refusal> {
    let mut scan = Scan {
        position: None,
        format: Format::Text,
        status: None,
        file: None,
        check: false,
    };
    let mut words = args.iter();
    while let Some(flag) = words.next() {
        let flag = flag.to_string_lossy().into_owned();
        match flag.as_str() {
            "--check" => scan.check = true,
            "--format" | "--status" | "--file" => {
                let value = words
                    .next()
                    .map(|value| value.to_string_lossy().into_owned());
                match flag.as_str() {
                    "--format" => {
                        let value = value.ok_or_else(|| {
                            usage(
                                code::FORMAT_CHOICE,
                                "--format needs `json`, `text` or `xml`",
                            )
                        })?;
                        scan.format = format::parse(&value)
                            .map_err(|problem| usage(code::FORMAT_CHOICE, problem))?;
                    }
                    "--status" => {
                        scan.status = Some(value.ok_or_else(|| {
                            usage(code::FLAG_NEEDS_VALUE, "--status needs a value")
                        })?)
                    }
                    _ => {
                        scan.file =
                            Some(value.ok_or_else(|| {
                                usage(code::FLAG_NEEDS_VALUE, "--file needs a value")
                            })?)
                    }
                }
            }
            _ if flag.starts_with("--") => {
                return Err(usage(
                    code::USAGE,
                    format!("{accepted} — unknown argument {flag}"),
                ));
            }
            _ => {
                if scan.position.replace(flag).is_some() {
                    return Err(usage(code::USAGE, accepted.to_owned()));
                }
            }
        }
    }
    Ok(scan)
}

/// Закрытый набор имён видов записей реестра и их каталоги.
pub(super) const KINDS: &[(&str, &str)] = &[
    ("thrust", "thrust"),
    ("slice", "slice"),
    ("work", "work"),
    ("obligation", "obligation"),
    ("limitation", "limitation"),
    ("upgrade", "upgrade"),
    ("invariant", "invariant"),
    ("review", "review"),
    ("decision", "adr"),
    ("specification", "rfc"),
];

/// Файлы записей каталога: имя файла без расширения — slug, текст — запись.
pub(crate) fn records(dir: &std::path::Path) -> Vec<(String, String)> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<std::path::PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "rs"))
        .collect();
    files.sort();
    files
        .into_iter()
        .filter_map(|path| {
            let id = path.file_stem()?.to_string_lossy().into_owned();
            let text = fs::read_to_string(&path).ok()?;
            Some((id, text))
        })
        .collect()
}

/// Число файлов каталога.
fn count_files(dir: &std::path::Path) -> usize {
    let Ok(entries) = fs::read_dir(dir) else {
        return 0;
    };
    entries
        .flatten()
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "rs"))
        .count()
}

/// Все записи реестра с их видом: (kind, id, text).
pub(super) fn all_records(context: &Context) -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    for (kind, dir) in KINDS {
        for (id, text) in records(
            &context
                .repo
                .root
                .join(format!("{}/{}", context.repo.config.doc, dir)),
        ) {
            out.push(((*kind).to_owned(), id, text));
        }
    }
    out
}

/// Единица работы карты и места в коде.
pub(super) struct Work {
    pub(super) id: String,
    pub(super) title: String,
    pub(super) slice: Option<String>,
    pub(super) taxon: Option<String>,
    pub(super) radius: Option<String>,
    pub(super) state: String,
}

/// Пустой перечень для отказа «запись не найдена».
pub(super) fn record_not_found(id: &str) -> Refusal {
    refused(
        code::RECORD_NOT_FOUND,
        format!("record {id} is not found in the registry"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Сбой отличается от законного отказа полем legitimate (RFC-0003): форма
    /// ответа доказана тестом, а не договорённостью.
    #[test]
    fn error_json_marks_the_failure_not_legitimate() {
        let answer = error_json("dacc: internal postcondition violated: x");
        assert!(answer.contains("\"schema\": \"dacc-error\""), "{answer}");
        assert!(answer.contains("\"legitimate\": false"), "{answer}");
        assert!(
            answer.contains("internal postcondition violated"),
            "{answer}"
        );
    }

    /// Ошибка разбора аргументов называет сам аргумент, а не литерал шаблона.
    #[test]
    fn map_error_names_the_argument() {
        let args = vec![OsString::from("--bogus")];
        let refusal = scan_args(&args, "map [--format json]")
            .err()
            .expect("отказ ожидается");
        assert!(refusal.reason().contains("--bogus"), "{}", refusal.reason());
    }
}
