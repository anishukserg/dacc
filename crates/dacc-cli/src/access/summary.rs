//! Сводки слоя доступа: `brief` — постоянное, `state` — изменяющееся за день.

use super::form::{json_string, STATE_LIMIT};
use super::{records, scan_args};
use crate::format::Format;
use crate::work::{stage_text, usage, Context, Refusal};
use crate::{code, record};
use std::ffi::OsString;

pub(super) fn brief(args: &[OsString]) -> Result<u8, Refusal> {
    let scan = scan_args(args, "brief [--format json]")?;
    if scan.position.is_some() || scan.status.is_some() || scan.file.is_some() || scan.check {
        return Err(usage(code::USAGE, "brief [--format json]"));
    }
    let context = Context::open()?;
    let root = &context.repo.root;
    let doc = &context.repo.config.doc;

    let thrusts: Vec<(String, String)> = records(&root.join(format!("{doc}/thrust")))
        .into_iter()
        .map(|(id, text)| (id, record::title(&text)))
        .collect();
    let open_slices: Vec<(String, String)> = records(&root.join(context.repo.config.slice_dir()))
        .into_iter()
        .filter(|(id, _)| !context.journal.closed_slices.contains_key(id))
        .map(|(id, text)| (id, record::title(&text)))
        .collect();
    let open_works: Vec<(String, String, String)> = context
        .works()?
        .into_iter()
        .filter(|(id, _)| !context.journal.stage(id).is_finished())
        .map(|(id, entry)| {
            let state = stage_text(context.journal.stage(&id)).to_owned();
            (id, state, entry.title)
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

pub(super) fn state(args: &[OsString]) -> Result<u8, Refusal> {
    let scan = scan_args(args, "state [--format json]")?;
    if scan.position.is_some() || scan.status.is_some() || scan.file.is_some() || scan.check {
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
        .map(|(id, entry)| {
            format!(
                "{id} [{}] {}",
                stage_text(context.journal.stage(&id)),
                entry.title
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
            for (index, line) in lines.iter().enumerate() {
                if out.len() + line.len() + 64 > STATE_LIMIT {
                    out.push_str(&format!("truncated: {index} of {total} lines\n"));
                    print!("{out}");
                    return Ok(0);
                }
                out.push_str(line);
                out.push('\n');
            }
            print!("{out}");
        }
    }
    Ok(0)
}
