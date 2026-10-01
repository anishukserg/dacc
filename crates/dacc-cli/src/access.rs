//! Слой доступа (решение 21, RFC-0003): карта реестра, место в коде и
//! директивы агентов. Ответы идут из дерева реестра и свёртки журнала, а не
//! из обхода каталогов и текстового поиска: навигация не требует find, grep
//! и ls.

use crate::format::{self, Format};
use crate::work::{quoted_after, refused, stage_text, usage, Context, Record, Refusal};
use crate::{code, work};
use std::ffi::OsString;
use std::fs;

/// `cargo dacc map [--format json]` — карта реестра одной командой.
pub fn run_map(args: &[OsString]) -> u8 {
    finish_json(peek_format(args), map(args))
}

/// `cargo dacc help [--format json]` — машинный контракт слоя доступа:
/// каждая команда объявляет стоимость и мутирование (RFC-0003).
pub fn run_help(args: &[OsString]) -> u8 {
    finish_json(peek_format(args), help(args))
}

/// Контракт команд слоя доступа: (команда, использование, стоимость,
/// мутирует).
const CONTRACT: &[(&str, &str, &str, bool)] = &[
    (
        "map",
        "map [--format json]",
        "read-only: the registry tree and the journal fold",
        false,
    ),
    (
        "where",
        "where --file <path> [--format json]",
        "read-only: one file and the registry",
        false,
    ),
    (
        "emit",
        "emit all [--check]",
        "writes AGENTS.md; --check is read-only",
        true,
    ),
    (
        "ls",
        "ls <kind> [--status <status>] [--format json]",
        "read-only: one registry directory",
        false,
    ),
    (
        "show",
        "show <id> [--format json]",
        "read-only: one record",
        false,
    ),
    (
        "refs",
        "refs <id> [--format json]",
        "read-only: the whole registry",
        false,
    ),
    (
        "find",
        "find <text> [--format json]",
        "read-only: the whole registry",
        false,
    ),
    (
        "brief",
        "brief [--format json]",
        "read-only: the plan and the journal fold",
        false,
    ),
    (
        "state",
        "state [--format json]",
        "read-only: the journal, capped at 2 KB",
        false,
    ),
    (
        "help",
        "help [--format json]",
        "read-only: this contract",
        false,
    ),
];

fn help(args: &[OsString]) -> Result<u8, Refusal> {
    let scan = scan_args(args, "help [--format json]")?;
    if scan.position.is_some() || scan.status.is_some() {
        return Err(usage(code::USAGE, "help [--format json]"));
    }
    match scan.format {
        Format::Xml => {
            return Err(usage(
                code::FORMAT_CHOICE,
                "xml is supported by map and where",
            ))
        }
        Format::Json => {
            let mut out = String::from("{\"schema\": \"dacc-help\", \"commands\": [");
            for (i, (command, accepted, cost, mutates)) in CONTRACT.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                out.push_str(&format!(
                    "{{\"command\": {}, \"usage\": {}, \"cost\": {}, \"mutates\": {}}}",
                    json_string(command),
                    json_string(accepted),
                    json_string(cost),
                    mutates
                ));
            }
            out.push_str("]}\n");
            println!("{out}");
        }
        Format::Text => {
            for (command, accepted, cost, mutates) in CONTRACT {
                let mutates = if *mutates { " [mutates]" } else { "" };
                println!("{command} {accepted} — {cost}{mutates}");
            }
        }
    }
    Ok(0)
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
                json_string(refusal.code()),
                json_string(refusal.reason())
            );
            refusal.exit_code()
        }
        Err(refusal) => work::finish(Err(refusal)),
    }
}

/// `cargo dacc where --file <path> [--format json]` — что известно о месте
/// в коде: его разметка, документы, ссылающиеся на неё, и связанные работы.
pub fn run_where(args: &[OsString]) -> u8 {
    finish_json(peek_format(args), where_file(args))
}

/// `cargo dacc emit all [--check]` — файлы-директивы агентов. Записанный файл
/// коммитится обычным ритуалом; `--check` сверяет свежесть без записи.
pub fn run_emit(args: &[OsString]) -> u8 {
    finish_json(peek_format(args), emit(args))
}

/// `cargo dacc ls <kind> [--status <status>] [--format json]` — перечень
/// записей реестра одного вида с их статусом или состоянием.
pub fn run_ls(args: &[OsString]) -> u8 {
    finish_json(peek_format(args), ls(args))
}

/// `cargo dacc refs <id> [--format json]` — граф записи: её ссылки и записи,
/// ссылающиеся на неё.
pub fn run_refs(args: &[OsString]) -> u8 {
    finish_json(peek_format(args), refs(args))
}

/// `cargo dacc brief [--format json]` — постоянная сводка: направления,
/// открытые срезы и открытые работы.
pub fn run_brief(args: &[OsString]) -> u8 {
    finish_json(peek_format(args), brief(args))
}

/// `cargo dacc state [--format json]` — изменяющееся за день: события дня и
/// открытая работа. Потолок 2 КБ, усечение явное (RFC-0003).
pub fn run_state(args: &[OsString]) -> u8 {
    finish_json(peek_format(args), state(args))
}

/// Потолок ответа state (RFC-0003): 2 КБ.
const STATE_LIMIT: usize = 2048;

fn brief(args: &[OsString]) -> Result<u8, Refusal> {
    let scan = scan_args(args, "brief [--format json]")?;
    if scan.position.is_some() || scan.status.is_some() {
        return Err(usage(code::USAGE, "brief [--format json]"));
    }
    let context = Context::open()?;
    let root = &context.repo.root;
    let doc = &context.repo.config.doc;

    let thrusts: Vec<(String, String)> = records_named(&root.join(format!("{doc}/thrust")))
        .into_iter()
        .map(|(id, text)| {
            let title = quoted_after(&text, "title: NonEmptyStr::new(").unwrap_or_default();
            (id, title)
        })
        .collect();
    let open_slices: Vec<(String, String)> =
        records_named(&root.join(context.repo.config.slice_dir()))
            .into_iter()
            .filter(|(id, _)| !context.journal.closed_slices.contains_key(id))
            .map(|(id, text)| {
                let title = quoted_after(&text, "title: NonEmptyStr::new(").unwrap_or_default();
                (id, title)
            })
            .collect();
    let open_works: Vec<(String, String, String)> = context
        .works()?
        .into_iter()
        .filter(|(id, _)| !context.journal.stage(id).is_finished())
        .map(|(id, record)| {
            let state = stage_text(context.journal.stage(&id)).to_owned();
            (id, state, record.title)
        })
        .collect();

    match scan.format {
        Format::Xml => {
            return Err(usage(
                code::FORMAT_CHOICE,
                "xml is supported by map and where",
            ))
        }
        Format::Json => {
            let mut out = String::from("{\"schema\": \"dacc-brief\", \"thrusts\": [");
            for (i, (id, title)) in thrusts.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                out.push_str(&format!(
                    "{{\"id\": {}, \"title\": {}}}",
                    json_string(id),
                    json_string(title)
                ));
            }
            out.push_str("], \"open_slices\": [");
            for (i, (id, title)) in open_slices.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                out.push_str(&format!(
                    "{{\"id\": {}, \"title\": {}}}",
                    json_string(id),
                    json_string(title)
                ));
            }
            out.push_str("], \"open_works\": [");
            for (i, (id, state, title)) in open_works.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                out.push_str(&format!(
                    "{{\"id\": {}, \"state\": {}, \"title\": {}}}",
                    json_string(id),
                    json_string(state),
                    json_string(title)
                ));
            }
            out.push_str("]}\n");
            println!("{out}");
        }
        Format::Text => {
            let mut out = String::new();
            if !thrusts.is_empty() {
                out.push_str("thrusts:\n");
                for (id, title) in &thrusts {
                    out.push_str(&format!("  {id} — {title}\n"));
                }
            }
            if !open_slices.is_empty() {
                out.push_str("open slices:\n");
                for (id, title) in &open_slices {
                    out.push_str(&format!("  {id} — {title}\n"));
                }
            }
            if open_works.is_empty() {
                out.push_str("no open works\n");
            } else {
                out.push_str("open works:\n");
                for (id, state, title) in &open_works {
                    out.push_str(&format!("  {id} [{state}] — {title}\n"));
                }
            }
            print!("{out}");
        }
    }
    Ok(0)
}

fn state(args: &[OsString]) -> Result<u8, Refusal> {
    let scan = scan_args(args, "state [--format json]")?;
    if scan.position.is_some() || scan.status.is_some() {
        return Err(usage(code::USAGE, "state [--format json]"));
    }
    let context = Context::open()?;
    let root = &context.repo.root;
    let config = &context.repo.config;

    // Изменяющееся за день — события с датой сегодняшнего дня (UTC) и
    // открытая работа, чьи состояния меняются каждый день.
    let now = dacc_journal::time::now();
    let today = now.get(..10).unwrap_or("").to_owned();
    let mut events = Vec::new();
    let file = root.join(config.journal_file());
    let dir = root.join(config.journal_dir());
    if let Ok((file_events, _)) = dacc_journal::read_file(&file) {
        events.extend(file_events);
    }
    if dir.is_dir() {
        if let Ok((dir_events, _)) = dacc_journal::read_dir(&dir) {
            events.extend(dir_events);
        }
    }
    let day: Vec<String> = events
        .iter()
        .filter(|event| event.at.starts_with(&today))
        .map(|event| format!("{} {} {}", event.at, event.kind.name(), event.subject.id()))
        .collect();
    let open: Vec<String> = context
        .works()?
        .into_iter()
        .filter(|(id, _)| !context.journal.stage(id).is_finished())
        .map(|(id, record)| {
            format!(
                "{id} [{}] {}",
                stage_text(context.journal.stage(&id)),
                record.title
            )
        })
        .collect();

    let mut lines: Vec<String> = Vec::new();
    lines.push(format!("date {today}"));
    lines.extend(day.iter().cloned());
    lines.push("open works:".to_owned());
    lines.extend(open.iter().cloned());

    match scan.format {
        Format::Xml => {
            return Err(usage(
                code::FORMAT_CHOICE,
                "xml is supported by map and where",
            ))
        }
        Format::Json => {
            let total = lines.len();
            let mut out = format!(
                "{{\"schema\": \"dacc-state\", \"date\": {}, \"lines\": [",
                json_string(&today)
            );
            let mut shown = 0;
            for (i, line) in lines.iter().enumerate() {
                let entry = format!("{}{}", if i > 0 { ", " } else { "" }, json_string(line));
                if out.len() + entry.len() + 64 > STATE_LIMIT {
                    out.push_str(&format!(
                        "], \"truncated\": true, \"shown\": {shown}, \"total\": {total}}}\n"
                    ));
                    println!("{out}");
                    return Ok(0);
                }
                out.push_str(&entry);
                shown += 1;
            }
            out.push_str(&format!(
                "], \"truncated\": false, \"shown\": {shown}, \"total\": {total}}}\n"
            ));
            println!("{out}");
        }
        Format::Text => {
            let total = lines.len();
            let mut out = String::new();
            let mut shown = total;
            for (index, line) in lines.iter().enumerate() {
                if out.len() + line.len() + 64 > STATE_LIMIT {
                    shown = index;
                    out.push_str(&format!("truncated: {shown} of {total} lines\n"));
                    print!("{out}");
                    return Ok(0);
                }
                out.push_str(line);
                out.push('\n');
            }
            let _ = shown;
            print!("{out}");
        }
    }
    Ok(0)
}

/// `cargo dacc find <text> [--format json]` — поиск по идентификаторам,
/// заголовкам и текстам записей реестра.
pub fn run_find(args: &[OsString]) -> u8 {
    finish_json(peek_format(args), find(args))
}

/// Все записи реестра с их видом: (kind, id, text).
fn all_records(context: &Context) -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    for (kind, dir) in KINDS {
        for (id, text) in records_named(
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

/// Ссылки вида `crate::модуль::идентификатор` в тексте записи.
fn crate_refs(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for segment in text.split("crate::").skip(1) {
        let end = segment
            .find([',', ')', '\n', '"', ' '])
            .unwrap_or(segment.len());
        let mut parts = segment[..end].split("::");
        if let (Some(module), Some(ident), None) = (parts.next(), parts.next(), parts.next()) {
            if !module.is_empty() && !ident.is_empty() {
                out.push((module.to_owned(), ident.to_owned()));
            }
        }
    }
    out
}

fn refs(args: &[OsString]) -> Result<u8, Refusal> {
    let scan = scan_args(args, "refs <id> [--format json]")?;
    let id = scan
        .position
        .ok_or_else(|| usage(code::USAGE, "refs <id> [--format json] is required"))?;
    let context = Context::open()?;
    let records = all_records(&context);
    let Some((_, _, text)) = records.iter().find(|(_, record_id, _)| *record_id == id) else {
        return Err(refused(
            code::RECORD_NOT_FOUND,
            format!("record {id} is not found in the registry"),
        ));
    };
    let ident = id.replace('-', "_");

    let mut outgoing: Vec<(String, String)> = Vec::new();
    for (module, target) in crate_refs(text) {
        if module == "anchor" {
            continue;
        }
        let slug = target.replace('_', "-");
        if slug != id
            && records.iter().any(|(_, record_id, _)| *record_id == slug)
            && !outgoing.iter().any(|(_, hit)| *hit == slug)
        {
            outgoing.push((module.clone(), slug));
        }
    }
    let mut incoming: Vec<(String, String)> = Vec::new();
    for (kind, record_id, record_text) in &records {
        if *record_id != id
            && record_text.contains(&format!("::{ident}"))
            && !incoming.iter().any(|(_, hit)| hit == record_id)
        {
            incoming.push((kind.clone(), record_id.clone()));
        }
    }

    match scan.format {
        Format::Xml => {
            return Err(usage(
                code::FORMAT_CHOICE,
                "xml is supported by map and where",
            ))
        }
        Format::Json => {
            let mut out = String::from("{\"schema\": \"dacc-refs\", \"id\": ");
            out.push_str(&json_string(&id));
            out.push_str(", \"outgoing\": [");
            for (i, (kind, target)) in outgoing.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                out.push_str(&format!(
                    "{{\"kind\": {}, \"id\": {}}}",
                    json_string(kind),
                    json_string(target)
                ));
            }
            out.push_str("], \"incoming\": [");
            for (i, (kind, source)) in incoming.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                out.push_str(&format!(
                    "{{\"kind\": {}, \"id\": {}}}",
                    json_string(kind),
                    json_string(source)
                ));
            }
            out.push_str("]}\n");
            println!("{out}");
        }
        Format::Text => {
            println!("record {id}");
            if outgoing.is_empty() && incoming.is_empty() {
                println!("no references");
            }
            for (kind, target) in &outgoing {
                println!("  -> {kind} {target}");
            }
            for (kind, source) in &incoming {
                println!("  <- {kind} {source}");
            }
        }
    }
    Ok(0)
}

fn find(args: &[OsString]) -> Result<u8, Refusal> {
    let scan = scan_args(args, "find <text> [--format json]")?;
    let query = scan
        .position
        .ok_or_else(|| usage(code::USAGE, "find <text> [--format json] is required"))?
        .to_lowercase();
    let context = Context::open()?;
    let mut matches: Vec<(String, String, String)> = Vec::new();
    for (kind, id, text) in all_records(&context) {
        let title = quoted_after(&text, "title: NonEmptyStr::new(").unwrap_or_default();
        if id.to_lowercase().contains(&query)
            || title.to_lowercase().contains(&query)
            || text.to_lowercase().contains(&query)
        {
            matches.push((kind, id, title));
        }
    }
    match scan.format {
        Format::Xml => {
            return Err(usage(
                code::FORMAT_CHOICE,
                "xml is supported by map and where",
            ))
        }
        Format::Json => {
            let mut out = String::from("{\"schema\": \"dacc-find\", \"matches\": [");
            for (i, (kind, id, title)) in matches.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                out.push_str(&format!(
                    "{{\"kind\": {}, \"id\": {}, \"title\": {}}}",
                    json_string(kind),
                    json_string(id),
                    json_string(title)
                ));
            }
            out.push_str("]}\n");
            println!("{out}");
        }
        Format::Text => {
            if matches.is_empty() {
                println!("no matches for {query:?}");
            } else {
                for (kind, id, title) in &matches {
                    println!("{kind} {id} — {title}");
                }
            }
        }
    }
    Ok(0)
}

/// `cargo dacc show <id> [--format json]` — запись реестра целиком.
pub fn run_show(args: &[OsString]) -> u8 {
    finish_json(peek_format(args), show(args))
}

/// Позиционный аргумент и флаги команды слоя доступа.
struct Scan {
    position: Option<String>,
    format: Format,
    status: Option<String>,
}

fn scan_args(args: &[OsString], accepted: &str) -> Result<Scan, Refusal> {
    let mut scan = Scan {
        position: None,
        format: Format::Text,
        status: None,
    };
    let mut words = args.iter();
    while let Some(flag) = words.next() {
        let flag = flag.to_string_lossy().into_owned();
        if flag == "--format" || flag == "--status" {
            let value = words
                .next()
                .map(|value| value.to_string_lossy().into_owned())
                .ok_or_else(|| usage(code::FLAG_NEEDS_VALUE, format!("{flag} needs a value")))?;
            if flag == "--format" {
                scan.format =
                    format::parse(&value).map_err(|problem| usage(code::FORMAT_CHOICE, problem))?;
            } else {
                scan.status = Some(value);
            }
        } else if flag.starts_with("--") {
            return Err(usage(
                code::USAGE,
                format!("{accepted} — unknown argument {flag}"),
            ));
        } else if scan.position.replace(flag).is_some() {
            return Err(usage(code::USAGE, accepted.to_owned()));
        }
    }
    Ok(scan)
}

/// Статус записи: `status: DocStatus::X` текста записи.
fn record_status(text: &str) -> Option<String> {
    let marker = "status: DocStatus::";
    let at = text.find(marker)?;
    let rest = &text[at + marker.len()..];
    let end = rest.find([',', ')', '\n']).unwrap_or(rest.len());
    Some(rest[..end].trim().to_owned())
}

fn ls(args: &[OsString]) -> Result<u8, Refusal> {
    let scan = scan_args(args, "ls <kind> [--status <status>] [--format json]")?;
    let kind = scan
        .position
        .ok_or_else(|| usage(code::USAGE, "ls <kind> [--status <status>] is required"))?;
    let Some((kind_name, dir)) = KINDS.iter().find(|(_, dir)| **dir == kind).copied() else {
        return Err(refused(
            code::KIND_UNKNOWN,
            format!("unknown kind {kind}: expected one of the registry directories"),
        ));
    };
    let context = Context::open()?;
    let root = &context
        .repo
        .root
        .join(format!("{}/{}", context.repo.config.doc, dir));
    let mut records: Vec<(String, String, String)> = Vec::new();
    for (id, text) in records_named(root) {
        let status = if dir == "work" {
            stage_text(context.journal.stage(&id)).to_owned()
        } else {
            record_status(&text).unwrap_or_else(|| "-".to_owned())
        };
        let keep = scan
            .status
            .as_deref()
            .is_none_or(|wanted| status.eq_ignore_ascii_case(wanted));
        if keep {
            let title = quoted_after(&text, "title: NonEmptyStr::new(").unwrap_or_default();
            records.push((id, status, title));
        }
    }
    match scan.format {
        Format::Xml => {
            return Err(usage(
                code::FORMAT_CHOICE,
                "xml is supported by map and where",
            ))
        }
        Format::Json => {
            let mut out = String::from("{\"schema\": \"dacc-ls\", \"kind\": ");
            out.push_str(&json_string(kind_name));
            out.push_str(", \"records\": [");
            for (i, (id, status, title)) in records.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                out.push_str(&format!(
                    "{{\"id\": {}, \"status\": {}, \"title\": {}}}",
                    json_string(id),
                    json_string(status),
                    json_string(title)
                ));
            }
            out.push_str("], \"truncated\": false}\n");
            println!("{out}");
        }
        Format::Text => {
            if records.is_empty() {
                println!("no records of kind {kind_name}");
            } else {
                for (id, status, title) in &records {
                    println!("{id}  {status}  {title}");
                }
            }
        }
    }
    Ok(0)
}

fn show(args: &[OsString]) -> Result<u8, Refusal> {
    let scan = scan_args(args, "show <id> [--format json]")?;
    let id = scan
        .position
        .ok_or_else(|| usage(code::USAGE, "show <id> [--format json] is required"))?;
    let context = Context::open()?;
    for (kind_name, dir) in KINDS {
        for (record_id, text) in records_named(
            &context
                .repo
                .root
                .join(format!("{}/{}", context.repo.config.doc, dir)),
        ) {
            if record_id != id {
                continue;
            }
            let status = if *dir == "work" {
                stage_text(context.journal.stage(&id)).to_owned()
            } else {
                record_status(&text).unwrap_or_else(|| "-".to_owned())
            };
            let title = quoted_after(&text, "title: NonEmptyStr::new(").unwrap_or_default();
            match scan.format {
                Format::Xml => {
                    return Err(usage(
                        code::FORMAT_CHOICE,
                        "xml is supported by map and where",
                    ))
                }
                Format::Json => println!(
                    "{{\"schema\": \"dacc-show\", \"kind\": {}, \"id\": {}, \"status\": {}, \"title\": {}, \"text\": {}}}",
                    json_string(kind_name),
                    json_string(&id),
                    json_string(&status),
                    json_string(&title),
                    json_string(&text)
                ),
                Format::Text => {
                    println!("{kind_name} {id} [{status}] {title}");
                    print!("{text}");
                }
            }
            return Ok(0);
        }
    }
    Err(refused(
        code::RECORD_NOT_FOUND,
        format!("record {id} is not found in the registry"),
    ))
}

/// Записи каталога: то же, что [`records`], для общих обращений.
fn records_named(dir: &std::path::Path) -> Vec<(String, String)> {
    records(dir)
}

fn emit(args: &[OsString]) -> Result<u8, Refusal> {
    let mut check = false;
    let mut choice: Option<String> = None;
    for arg in args {
        let arg = arg.to_string_lossy().into_owned();
        if arg == "--check" {
            check = true;
        } else if choice.replace(arg.clone()).is_some() {
            return Err(usage(code::USAGE, "emit all [--check]"));
        }
    }
    if choice.as_deref() != Some("all") {
        return Err(usage(code::USAGE, "emit all [--check]"));
    }
    let context = Context::open()?;
    let text = directives(&context.repo.config);
    let path = context.repo.root.join("AGENTS.md");
    if check {
        let committed = fs::read_to_string(&path).unwrap_or_default();
        if committed != text {
            return Err(refused(
                code::DIRECTIVES_STALE,
                "AGENTS.md is stale: run cargo dacc emit all",
            ));
        }
        println!("AGENTS.md is fresh");
        return Ok(0);
    }
    fs::write(&path, text).map_err(|error| {
        refused(
            code::FILE_NOT_READ,
            format!("AGENTS.md not written: {error}"),
        )
    })?;
    println!("AGENTS.md written");
    Ok(0)
}

/// Текст директивы агентов: навигация, ритуалы и что отвечают команды.
/// Правила коммитов продукта берутся из настройки, а не повторяются руками.
fn directives(config: &crate::config::Config) -> String {
    let mut out = String::from(
        "# Agent directives\n\n\
         Generated by `cargo dacc emit all`; `cargo dacc emit all --check` verifies\n\
         this file is fresh. Do not edit it by hand — the generator is the source.\n\n\
         ## Navigation\n\n\
         - `cargo dacc map [--format json]` — the registry map: thrusts, slices,\n\
         \x20 works with journal states, and counters.\n\
         - `cargo dacc where --file <path> [--format json]` — a place in code: its\n\
         \x20 `#[doc_anchor]` markup, the documents citing it, the related works.\n\
         - `cargo dacc work state [<w-slug>]` and `cargo dacc metrics` — work states\n\
         \x20 and counters from the journal fold.\n\
         - Do not navigate the registry with find, grep or ls; ripgrep is allowed\n\
         \x20 for sources only.\n\n\
         ## Rituals\n\n\
         - Every change has a basis in the plan: a work item with an origin.\n\
         - Commits go through `cargo dacc commit -F <message-file> -- <paths>`: the\n\
         \x20 subject is `[TYPE](scope): summary`, and the trailer is\n\
         \x20 `Dacc-Work: w-slug`.\n\
         - `cargo dacc work land <w-slug>` lands with a proof on the tree; a defect\n\
         \x20 fix — the Divergence origin — or an irreversible change lands only with\n\
         \x20 `--red-before <test>`; `--mutation-proof <check>` and\n\
         \x20 `--anti-vacuum <subject>` state the other anti-vacuous proofs.\n\
         - A slice closes on its outcome: `cargo dacc slice close <s-slug>`.\n\n\
         ## Product settings\n\n",
    );
    out.push_str(&format!(
        "- Commit types: {}.\n",
        config.commit_types.join(", ")
    ));
    out.push_str(&format!(
        "- Subject limit: {} characters.\n",
        config.subject_limit
    ));
    out.push_str(&format!("- Commit rules: {}.\n", config.commit_rules));
    out
}

fn where_file(args: &[OsString]) -> Result<u8, Refusal> {
    let mut format = Format::Text;
    let mut file: Option<String> = None;
    let mut words = args.iter();
    while let Some(flag) = words.next() {
        let flag = flag.to_string_lossy().into_owned();
        let value = words
            .next()
            .map(|value| value.to_string_lossy().into_owned())
            .ok_or_else(|| usage(code::FLAG_NEEDS_VALUE, format!("{flag} needs a value")))?;
        match flag.as_str() {
            "--file" => file = Some(value),
            "--format" => {
                format =
                    format::parse(&value).map_err(|problem| usage(code::FORMAT_CHOICE, problem))?
            }
            other => {
                return Err(usage(
                    code::USAGE,
                    format!("where --file <path> [--format json] — unknown argument {other}"),
                ))
            }
        }
    }
    let file = file.ok_or_else(|| usage(code::USAGE, "where --file <path> is required"))?;
    let context = Context::open()?;
    let text = fs::read_to_string(context.repo.root.join(&file))
        .map_err(|_| refused(code::FILE_NOT_READ, format!("file {file} cannot be read")))?;

    // Разметка места — сама по себе ответ; без неё отдельная строка, а не
    // пустой успех (анти-вакуум).
    let anchors = anchors_in(&text);
    let mut documents = Vec::new();
    for (kind, dir) in KINDS {
        for (id, record) in records(
            &context
                .repo
                .root
                .join(format!("{}/{}", context.repo.config.doc, dir)),
        ) {
            let referenced = anchors.iter().any(|anchor| {
                record.contains(&format!("[{anchor}]")) || record.contains(&anchor_ident(anchor))
            });
            if referenced {
                documents.push(Document {
                    title: quoted_after(&record, "title: NonEmptyStr::new(").unwrap_or_default(),
                    kind: (*kind).to_owned(),
                    id,
                });
            }
        }
    }
    let mut works = Vec::new();
    for (id, record) in records(&context.repo.root.join(context.repo.config.work_dir())) {
        let referenced = documents
            .iter()
            .any(|document| record.contains(&document.id.replace('-', "_")))
            || anchors
                .iter()
                .any(|anchor| record.contains(&anchor_ident(anchor)));
        if referenced {
            works.push(Work {
                state: stage_text(context.journal.stage(&id)).to_owned(),
                slice: ref_after(&record, "slice: crate::slice::"),
                taxon: ref_after(&record, "taxon: taxon!(Subsystem, ")
                    .map(|name| name.to_lowercase()),
                radius: ref_after(&record, "radius: BlastRadius::").map(|r| r.to_lowercase()),
                title: quoted_after(&record, "title: NonEmptyStr::new(").unwrap_or_default(),
                id,
            });
        }
    }

    match format {
        Format::Json => println!("{}", where_json(&file, &anchors, &documents, &works)),
        Format::Text => print!("{}", where_text(&file, &anchors, &documents, &works)),
        Format::Xml => println!("{}", where_xml(&file, &anchors, &documents, &works)),
    }
    Ok(0)
}

/// Идентификатор якоря в ссылках Rust: `plan-ir` → `anchor::plan_ir`.
fn anchor_ident(anchor: &str) -> String {
    format!("anchor::{}", anchor.replace('-', "_"))
}

/// Имена видов записей реестра и их каталоги.
const KINDS: &[(&str, &str)] = &[
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

/// Документ, ссылающийся на якорь места.
struct Document {
    id: String,
    kind: String,
    title: String,
}

/// Идентификаторы `doc_anchor(id = "…")` файла.
fn anchors_in(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(at) = rest.find("doc_anchor(") {
        let end = rest[at..]
            .find(")]")
            .map(|close| at + close)
            .unwrap_or(rest.len());
        let segment = &rest[at..end];
        if let Some(id_at) = segment.find("id = \"") {
            let after = &segment[id_at + 6..];
            if let Some(close) = after.find('"') {
                out.push(after[..close].to_owned());
            }
        }
        rest = &rest[end..];
    }
    out
}

fn where_text(file: &str, anchors: &[String], documents: &[Document], works: &[Work]) -> String {
    if anchors.is_empty() {
        return format!("no doc_anchor markup in {file}\n");
    }
    let mut out = format!("file {file}\nanchors: {}\n", anchors.join(", "));
    if !documents.is_empty() {
        out.push_str("documents:\n");
        for document in documents {
            out.push_str(&format!(
                "  {} {} — {}\n",
                document.kind, document.id, document.title
            ));
        }
    }
    if !works.is_empty() {
        out.push_str("works:\n");
        for work in works {
            out.push_str(&format!(
                "  {} [{}] — {}\n",
                work.id, work.state, work.title
            ));
        }
    }
    out
}

fn where_json(file: &str, anchors: &[String], documents: &[Document], works: &[Work]) -> String {
    let mut out = format!(
        "{{\"schema\": \"dacc-where\", \"file\": {}, \"anchors\": [",
        json_string(file)
    );
    for (i, anchor) in anchors.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push_str(&json_string(anchor));
    }
    out.push_str("], \"documents\": [");
    for (i, document) in documents.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push_str(&format!(
            "{{\"kind\": {}, \"id\": {}, \"title\": {}}}",
            json_string(&document.kind),
            json_string(&document.id),
            json_string(&document.title)
        ));
    }
    out.push_str("], \"works\": [");
    for (i, work) in works.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push_str(&format!(
            "{{\"id\": {}, \"state\": {}, \"title\": {}}}",
            json_string(&work.id),
            json_string(&work.state),
            json_string(&work.title)
        ));
    }
    out.push_str("]}\n");
    out
}

/// Направление карты.
struct Thrust {
    id: String,
    title: String,
    outcome: String,
}

/// Срез карты.
struct Slice {
    id: String,
    title: String,
    outcome: String,
    thrust: String,
    closed: bool,
}

/// Единица работы карты.
struct Work {
    id: String,
    title: String,
    slice: Option<String>,
    taxon: Option<String>,
    radius: Option<String>,
    state: String,
}

fn map(args: &[OsString]) -> Result<u8, Refusal> {
    let format = scan_format(args)?;
    let context = Context::open()?;
    let root = &context.repo.root;

    let mut thrusts = Vec::new();
    for (id, text) in records(&root.join(format!("{}/thrust", context.repo.config.doc))) {
        thrusts.push(Thrust {
            outcome: quoted_after(&text, "outcome: NonEmptyStr::new(").unwrap_or_default(),
            title: quoted_after(&text, "title: NonEmptyStr::new(").unwrap_or_default(),
            id,
        });
    }

    let mut slices = Vec::new();
    for (id, text) in records(&root.join(context.repo.config.slice_dir())) {
        let record = Record::parse(&text);
        slices.push(Slice {
            closed: context.journal.closed_slices.contains_key(&id),
            thrust: ref_after(&text, "thrust: crate::thrust::").unwrap_or_default(),
            outcome: quoted_after(&text, "outcome: NonEmptyStr::new(").unwrap_or_default(),
            title: record.title,
            id,
        });
    }

    let mut works = Vec::new();
    for (id, record) in context.works()? {
        works.push(Work {
            state: stage_text(context.journal.stage(&id)).to_owned(),
            slice: record.slice.clone(),
            taxon: record.area.clone(),
            radius: record.radius.clone(),
            title: record.title,
            id,
        });
    }

    let decisions = count_files(&root.join(format!("{}/adr", context.repo.config.doc)));
    let specifications = count_files(&root.join(format!("{}/rfc", context.repo.config.doc)));

    match format {
        Format::Json => {
            println!(
                "{}",
                cap_json_map(
                    render_json(&thrusts, &slices, &works, decisions, specifications),
                    MAP_LIMIT
                )
            );
        }
        Format::Text => {
            print!(
                "{}",
                cap_text(
                    &render_text(&thrusts, &slices, &works, decisions, specifications),
                    MAP_LIMIT
                )
            );
        }
        Format::Xml => {
            println!(
                "{}",
                render_xml(&thrusts, &slices, &works, decisions, specifications)
            );
        }
    }
    Ok(0)
}

/// `--format` из аргументов; лишнее — отказ с перечнем принятого.
fn scan_format(args: &[OsString]) -> Result<Format, Refusal> {
    let mut format = Format::Text;
    let mut words = args.iter();
    while let Some(flag) = words.next() {
        let flag = flag.to_string_lossy();
        if flag != "--format" {
            return Err(usage(
                code::USAGE,
                "map [--format json] — unknown argument {flag}",
            ));
        }
        let value = words
            .next()
            .ok_or_else(|| usage(code::FORMAT_CHOICE, "--format needs `json` or `text`"))?;
        format = format::parse(&value.to_string_lossy())
            .map_err(|problem| usage(code::FORMAT_CHOICE, problem))?;
    }
    Ok(format)
}

/// Файлы записей каталога: имя файла без расширения — slug, текст — запись.
fn records(dir: &std::path::Path) -> Vec<(String, String)> {
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

/// Имя файла из числа файлов каталога.
fn count_files(dir: &std::path::Path) -> usize {
    let Ok(entries) = fs::read_dir(dir) else {
        return 0;
    };
    entries
        .flatten()
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "rs"))
        .count()
}

/// Идентификатор после маркера ссылки: `thrust: crate::thrust::t-001`.
fn ref_after(text: &str, marker: &str) -> Option<String> {
    let at = text.find(marker)?;
    let rest = &text[at + marker.len()..];
    let end = rest.find([',', ')', '\n']).unwrap_or(rest.len());
    let slug = rest[..end].trim();
    (!slug.is_empty()).then(|| slug.to_owned())
}

/// Карта текстом: дерево направлений, срезов и работ и счётчики.
fn render_text(
    thrusts: &[Thrust],
    slices: &[Slice],
    works: &[Work],
    decisions: usize,
    specifications: usize,
) -> String {
    let mut out = String::new();
    for thrust in thrusts {
        out.push_str(&format!("{} — {}\n", thrust.id, thrust.title));
        out.push_str(&format!("    {}\n", thrust.outcome));
        for slice in slices.iter().filter(|slice| slice.thrust == thrust.id) {
            let state = if slice.closed { "closed" } else { "open" };
            out.push_str(&format!("  {} [{}] — {}\n", slice.id, state, slice.title));
            out.push_str(&format!("      {}\n", slice.outcome));
            for work in works
                .iter()
                .filter(|work| work.slice.as_deref() == Some(slice.id.as_str()))
            {
                out.push_str(&format!(
                    "    {} [{}] — {}\n",
                    work.id, work.state, work.title
                ));
            }
        }
    }
    let open = slices.iter().filter(|slice| !slice.closed).count();
    out.push_str(&format!(
        "thrusts: {}, slices: {} (open {}, closed {}), works: {}\n",
        thrusts.len(),
        slices.len(),
        open,
        slices.len() - open,
        works.len()
    ));
    let mut stages: Vec<String> = Vec::new();
    for state in ["planned", "started", "landed", "abandoned"] {
        let count = works.iter().filter(|work| work.state == state).count();
        if count > 0 {
            stages.push(format!("{state} {count}"));
        }
    }
    out.push_str(&format!("works by state: {}\n", stages.join(", ")));
    out.push_str(&format!(
        "decisions: {decisions}, specifications: {specifications}\n"
    ));
    out
}

/// Карта машинным форматом: те же данные полями.
fn render_json(
    thrusts: &[Thrust],
    slices: &[Slice],
    works: &[Work],
    decisions: usize,
    specifications: usize,
) -> String {
    let mut out = String::from("{\"schema\": \"dacc-map\", \"thrusts\": [");
    for (i, thrust) in thrusts.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push_str(&format!(
            "{{\"id\": {}, \"title\": {}, \"outcome\": {}}}",
            json_string(&thrust.id),
            json_string(&thrust.title),
            json_string(&thrust.outcome)
        ));
    }
    out.push_str("], \"slices\": [");
    for (i, slice) in slices.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push_str(&format!(
            "{{\"id\": {}, \"title\": {}, \"thrust\": {}, \"closed\": {}, \"outcome\": {}}}",
            json_string(&slice.id),
            json_string(&slice.title),
            json_string(&slice.thrust),
            slice.closed,
            json_string(&slice.outcome)
        ));
    }
    out.push_str("], \"works\": [");
    for (i, work) in works.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push_str(&format!(
            "{{\"id\": {}, \"title\": {}, \"slice\": {}, \"taxon\": {}, \"radius\": {}, \"state\": {}}}",
            json_string(&work.id),
            json_string(&work.title),
            work.slice
                .as_deref()
                .map_or_else(|| "null".to_owned(), json_string),
            work.taxon
                .as_deref()
                .map_or_else(|| "null".to_owned(), json_string),
            work.radius
                .as_deref()
                .map_or_else(|| "null".to_owned(), json_string),
            json_string(&work.state)
        ));
    }
    out.push_str(&format!(
        "], \"decisions\": {decisions}, \"specifications\": {specifications}}}\n"
    ));
    out
}

/// Строка JSON с экранированием управляющих символов.
fn json_string(text: &str) -> String {
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Строка XML с экранированием разметки в атрибутах.
fn xml_string(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        match c {
            '&' => out.push_str(concat!("&", "amp;")),
            '<' => out.push_str(concat!("&", "lt;")),
            '>' => out.push_str(concat!("&", "gt;")),
            '"' => out.push_str(concat!("&", "quot;")),
            c => out.push(c),
        }
    }
    out
}

/// Потолок ответа map (RFC-0003): 8 КБ независимо от размера проекта.
const MAP_LIMIT: usize = 8192;

/// Явное усечение текста до потолка: ответ не превышает limit, и усечение
/// названо строкой, а не молчанием.
fn cap_text(text: &str, limit: usize) -> String {
    if text.len() <= limit {
        return text.to_owned();
    }
    let total = text.lines().count();
    let mut out = String::new();
    for (index, line) in text.lines().enumerate() {
        if out.len() + line.len() + 64 > limit {
            out.push_str(&format!("truncated: {index} of {total} lines\n"));
            return out;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

/// Явное усечение JSON карты: массив работ вырезается целиком и поле truncated
/// названо явно; крайний случай — минимальный ответ со счётчиками.
fn cap_json_map(text: String, limit: usize) -> String {
    if text.len() <= limit {
        return text;
    }
    if let (Some(start), Some(end)) = (text.find("\"works\": ["), text.find("], \"decisions\"")) {
        let mut out = String::with_capacity(limit);
        out.push_str(&text[..start]);
        out.push_str("\"works\": []");
        out.push_str(&text[end + 1..]);
        if let Some(close) = out.rfind("}\n") {
            out.insert_str(close, ", \"truncated\": true");
        }
        if out.len() <= limit {
            return out;
        }
    }
    "{\"schema\": \"dacc-map\", \"truncated\": true}\n".to_owned()
}

/// Карта версионированным контрактом XML (RFC-0003): тот же потолок и явное
/// усечение в элементе counts.
fn render_xml(
    thrusts: &[Thrust],
    slices: &[Slice],
    works: &[Work],
    decisions: usize,
    specifications: usize,
) -> String {
    let mut entries: Vec<String> = Vec::new();
    for thrust in thrusts {
        entries.push(format!(
            "  <thrust id=\"{}\" title=\"{}\"/>\n",
            xml_string(&thrust.id),
            xml_string(&thrust.title)
        ));
    }
    for slice in slices {
        entries.push(format!(
            "  <slice id=\"{}\" title=\"{}\" thrust=\"{}\" closed=\"{}\"/>\n",
            xml_string(&slice.id),
            xml_string(&slice.title),
            xml_string(&slice.thrust),
            slice.closed
        ));
    }
    for work in works {
        entries.push(format!(
            "  <work id=\"{}\" title=\"{}\" state=\"{}\"/>\n",
            xml_string(&work.id),
            xml_string(&work.title),
            xml_string(&work.state)
        ));
    }
    let total = entries.len();
    let head = String::from("<map schema=\"dacc-map\" version=\"1\">\n");
    let mut body = String::new();
    let mut shown = 0;
    for (index, entry) in entries.iter().enumerate() {
        if head.len() + body.len() + entry.len() + 128 > MAP_LIMIT {
            break;
        }
        body.push_str(entry);
        shown = index + 1;
    }
    format!(
        "{head}{body}  <counts decisions=\"{decisions}\" specifications=\"{specifications}\" shown=\"{shown}\" total=\"{total}\" truncated=\"{}\"/>\n</map>\n",
        shown < total
    )
}

/// Ответ where версионированным контрактом XML.
fn where_xml(file: &str, anchors: &[String], documents: &[Document], works: &[Work]) -> String {
    let mut out = format!(
        "<where schema=\"dacc-where\" version=\"1\" file=\"{}\">\n",
        xml_string(file)
    );
    if anchors.is_empty() {
        out.push_str("  <no_markup/>\n</where>\n");
        return out;
    }
    for anchor in anchors {
        out.push_str(&format!("  <anchor id=\"{}\"/>\n", xml_string(anchor)));
    }
    for document in documents {
        out.push_str(&format!(
            "  <document kind=\"{}\" id=\"{}\" title=\"{}\"/>\n",
            xml_string(&document.kind),
            xml_string(&document.id),
            xml_string(&document.title)
        ));
    }
    for work in works {
        out.push_str(&format!(
            "  <work id=\"{}\" state=\"{}\" title=\"{}\"/>\n",
            xml_string(&work.id),
            xml_string(&work.state),
            xml_string(&work.title)
        ));
    }
    out.push_str("</where>\n");
    out
}
