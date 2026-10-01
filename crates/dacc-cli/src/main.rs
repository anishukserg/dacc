//! `cargo dacc` — правила коммитов, калитка, хуки и журнал DACC
//! (решения 8, 14 и 15).
//!
//! ```text
//! cargo dacc commit -F <message> [--log <file>] [--timeout <seconds>] -- <paths…>
//! cargo dacc msg-check [--form-only] <message> | --range <range>
//! cargo dacc gate [--repo <directory>] [--journal-only] [<tree>]
//! cargo dacc hook pre-commit | commit-msg <message> | pre-push <remote> <url>
//! cargo dacc hooks install [--force]
//! cargo dacc work start | land | drop | state …
//! cargo dacc slice close <s-slug>
//! cargo dacc journal hash [<revision>] | import --work <w-slug> [--close-finished-slices]
//! ```
//!
//! Соглашения, которым подлежит настройка продукта, читает модуль `config` из
//! `dacc.toml`; то, что настройке не подлежит, собрано в модуле `layout`.
//! Без файла настройки поведение — прежнее (решение 20). Внешних зависимостей
//! нет: хук собирает инструмент, и сборка не тянет граф крейтов.

// Нестабильные возможности запрещены и в doctest: lints манифеста на них не
// распространяются, а атаки выполняются под RUSTC_BOOTSTRAP (решение 12).
#![doc(test(attr(forbid(unstable_features))))]

mod access;
mod code;
mod commit;
mod config;
mod format;
mod gate;
mod git;
mod hooks;
mod journal;
mod layout;
mod message;
mod proof;
mod work;
mod work_new;

use std::ffi::OsString;
use std::process::ExitCode;

const USAGE: &str = "cargo dacc — commit rules and the DACC journal (decisions 8, 14 and 15)

  cargo dacc commit -F <message> [--log <file>] [--timeout <seconds>] -- <paths…>
  cargo dacc msg-check [--form-only] <message> | --range <range>
  cargo dacc map [--format json]
  cargo dacc where --file <path> [--format json]
  cargo dacc gate [--repo <directory>] [--journal-only] [<tree>]
  cargo dacc hook pre-commit | commit-msg <message> | pre-push <remote> <url>
  cargo dacc hooks install [--force]
  cargo dacc work start <w-slug> | land <w-slug> [--commit <revision>] | drop <w-slug> --reason <reason> | state [<w-slug>]
  cargo dacc slice close <s-slug>
  cargo dacc journal hash [<revision>]
  cargo dacc journal import --work <w-slug> [--close-finished-slices]
  cargo dacc metrics
  cargo dacc upgrade [<from>] <to>

  The work, slice and journal import commands accept --trailer <trailer> for
  extra trailer lines of the commit message.";

fn main() -> ExitCode {
    let mut args: Vec<OsString> = std::env::args_os().skip(1).collect();
    // `cargo dacc …` запускает `cargo-dacc dacc …`.
    if args.first().and_then(|a| a.to_str()) == Some("dacc") {
        args.remove(0);
    }
    let Some(command) = args.first().map(|a| a.to_string_lossy().into_owned()) else {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    };
    let rest = &args[1..];
    let code = match command.as_str() {
        "commit" => commit::run(rest),
        "map" => access::run_map(rest),
        "where" => access::run_where(rest),
        "msg-check" => message::run(rest),
        "gate" => gate::run(rest),
        "hook" => hooks::run(rest),
        "hooks" => hooks::install(rest),
        "work" => work::run_work(rest),
        "slice" => work::run_slice(rest),
        "journal" => journal::run(rest),
        "metrics" => work::run_metrics(rest),
        "upgrade" => work::run_upgrade(rest),
        "help" | "--help" | "-h" => {
            println!("{USAGE}");
            0
        }
        other => {
            eprintln!("unknown command {other}\n\n{USAGE}");
            2
        }
    };
    ExitCode::from(code)
}
