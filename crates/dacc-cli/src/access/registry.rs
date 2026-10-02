//! Перечень и чтение записей реестра: `ls`, `show`, `refs` и `find` —
//! ответы из дерева реестра, без обхода каталогов и текстового поиска.

use super::form::{cap_text, json_string, MAP_LIMIT};
use super::{all_records, record_not_found, records, scan_args};
use crate::format::Format;
use crate::work::{refused, stage_text, usage, Context, Refusal};
use crate::{code, record};
use std::ffi::OsString;

pub(super) fn ls(args: &[OsString]) -> Result<u8, Refusal> {
    let scan = scan_args(args, "ls <kind> [--status <status>] [--format json]")?;
    let kind = scan
        .position
        .ok_or_else(|| usage(code::USAGE, "ls <kind> [--status <status>] is required"))?;
    let Some((kind_name, dir)) = super::KINDS.iter().find(|(_, dir)| **dir == kind).copied() else {
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
    let mut entries: Vec<(String, String, String)> = Vec::new();
    for (id, text) in records(root) {
        let status = if dir == "work" {
            stage_text(context.journal.stage(&id)).to_owned()
        } else {
            record::doc_status(&text).unwrap_or_else(|| "-".to_owned())
        };
        let keep = scan
            .status
            .as_deref()
            .is_none_or(|wanted| status.eq_ignore_ascii_case(wanted));
        if keep {
            entries.push((id, status, record::title(&text)));
        }
    }
    match scan.format {
        Format::Xml => {
            return Err(usage(
                code::FORMAT_CHOICE,
                "xml is supported by map and where",
            ))
        }
        Format::Json => println!("{}", ls_json(kind_name, &entries)),
        Format::Text => print!("{}", cap_text(&ls_text(kind_name, &entries), MAP_LIMIT)),
    }
    Ok(0)
}

/// Перечень машинным форматом: усечение до сериализации — записи добавляются
/// до потолка MAP_LIMIT, и поле truncated соответствует усечению, называя
/// показанное и полное число (работа w-access-answers).
fn ls_json(kind_name: &str, records: &[(String, String, String)]) -> String {
    let mut out = String::from("{\"schema\": \"dacc-ls\", \"kind\": ");
    out.push_str(&json_string(kind_name));
    out.push_str(", \"records\": [");
    let total = records.len();
    let mut shown = 0;
    for (index, (id, status, title)) in records.iter().enumerate() {
        let entry = format!(
            "{}{{\"id\": {}, \"status\": {}, \"title\": {}}}",
            if index > 0 { ", " } else { "" },
            json_string(id),
            json_string(status),
            json_string(title)
        );
        if out.len() + entry.len() + 64 > MAP_LIMIT {
            break;
        }
        out.push_str(&entry);
        shown += 1;
    }
    out.push_str(&format!(
        "], \"shown\": {shown}, \"total\": {total}, \"truncated\": {}}}\n",
        shown < total
    ));
    out
}

/// Перечень текстом в порядке записей; потолок ответа держит cap_text.
fn ls_text(kind_name: &str, records: &[(String, String, String)]) -> String {
    if records.is_empty() {
        return format!("no records of kind {kind_name}\n");
    }
    let mut out = String::new();
    for (id, status, title) in records {
        out.push_str(&format!("{id}  {status}  {title}\n"));
    }
    out
}

pub(super) fn show(args: &[OsString]) -> Result<u8, Refusal> {
    let scan = scan_args(args, "show <id> [--format json]")?;
    let id = scan
        .position
        .ok_or_else(|| usage(code::USAGE, "show <id> [--format json] is required"))?;
    let context = Context::open()?;
    for (kind_name, dir) in super::KINDS {
        for (record_id, text) in records(
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
                record::doc_status(&text).unwrap_or_else(|| "-".to_owned())
            };
            let title = record::title(&text);
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
    Err(record_not_found(&id))
}

pub(super) fn refs(args: &[OsString]) -> Result<u8, Refusal> {
    let scan = scan_args(args, "refs <id> [--format json]")?;
    let id = scan
        .position
        .ok_or_else(|| usage(code::USAGE, "refs <id> [--format json] is required"))?;
    let context = Context::open()?;
    let entries = all_records(&context);
    let Some((_, _, text)) = entries.iter().find(|(_, record_id, _)| *record_id == id) else {
        return Err(record_not_found(&id));
    };
    let ident = id.replace('-', "_");

    let mut outgoing: Vec<(String, String)> = Vec::new();
    for (module, target) in crate_refs(text) {
        if module == "anchor" {
            continue;
        }
        let slug = target.replace('_', "-");
        if slug != id
            && entries.iter().any(|(_, record_id, _)| *record_id == slug)
            && !outgoing.iter().any(|(_, hit)| *hit == slug)
        {
            outgoing.push((module.clone(), slug));
        }
    }
    let mut incoming: Vec<(String, String)> = Vec::new();
    for (kind, record_id, record_text) in &entries {
        if *record_id != id
            && mentions(record_text, &format!("::{ident}"))
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

/// Вхождение в тексте записи без комментариев: упоминание в комментарии не
/// связывает записи (работа w-record-parsing-hardening).
fn mentions(text: &str, needle: &str) -> bool {
    record::strip_comments(text).contains(needle)
}

/// Ссылки вида `crate::модуль::идентификатор` в тексте записи.
fn crate_refs(text: &str) -> Vec<(String, String)> {
    let text = record::strip_comments(text);
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

pub(super) fn find(args: &[OsString]) -> Result<u8, Refusal> {
    let scan = scan_args(args, "find <text> [--format json]")?;
    let query = scan
        .position
        .ok_or_else(|| usage(code::USAGE, "find <text> [--format json] is required"))?
        .to_lowercase();
    let context = Context::open()?;
    let mut matches: Vec<(String, String, String)> = Vec::new();
    for (kind, id, text) in all_records(&context) {
        let title = record::title(&text);
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Поле truncated перечня соответствует усечению: показанное и полное
    /// число названы явно, и потолок ответа держится.
    #[test]
    fn ls_truncation_is_truthful() {
        let records: Vec<(String, String, String)> = (0..400)
            .map(|index| {
                (
                    format!("w-{index:04}"),
                    "planned".to_owned(),
                    format!("Работа {index}"),
                )
            })
            .collect();
        let out = ls_json("work", &records);
        assert!(out.len() <= MAP_LIMIT + 64, "{}", out.len());
        assert!(out.contains("\"truncated\": true"), "{out}");
        assert!(out.contains("\"total\": 400"), "{out}");
        assert!(out.contains("w-0000"), "{out}");
        assert!(!out.contains("w-0399"), "{out}");
        let small = ls_json("work", &records[..1]);
        assert!(
            small.contains("\"shown\": 1, \"total\": 1, \"truncated\": false"),
            "{small}"
        );
    }
}
