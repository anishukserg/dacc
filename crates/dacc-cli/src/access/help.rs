//! `cargo dacc help [--format json]` — машинный контракт слоя доступа:
//! каждая команда объявляет использование, стоимость и мутирование (RFC-0003).

use super::form::json_string;
use super::scan_args;
use crate::code;
use crate::format::Format;
use crate::work::{usage, Refusal};
use std::ffi::OsString;

/// Контракт команд слоя доступа: (команда, использование, стоимость,
/// мутирование).
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

pub(super) fn run(args: &[OsString]) -> Result<u8, Refusal> {
    let scan = scan_args(args, "help [--format json]")?;
    if scan.position.is_some() || scan.status.is_some() || scan.file.is_some() || scan.check {
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
