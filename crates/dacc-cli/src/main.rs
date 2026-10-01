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
  cargo dacc emit all [--check]
  cargo dacc ls <kind> [--status <status>] [--format json]
  cargo dacc show <id> [--format json]
  cargo dacc refs <id> [--format json]
  cargo dacc find <text> [--format json]
  cargo dacc brief [--format json]
  cargo dacc state [--format json]
  cargo dacc help [--format json]
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
    install_failure_report();
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
        "emit" => access::run_emit(rest),
        "ls" => access::run_ls(rest),
        "show" => access::run_show(rest),
        "refs" => access::run_refs(rest),
        "find" => access::run_find(rest),
        "brief" => access::run_brief(rest),
        "state" => access::run_state(rest),
        "msg-check" => message::run(rest),
        "gate" => gate::run(rest),
        "hook" => hooks::run(rest),
        "hooks" => hooks::install(rest),
        "work" => work::run_work(rest),
        "slice" => work::run_slice(rest),
        "journal" => journal::run(rest),
        "metrics" => work::run_metrics(rest),
        "upgrade" => work::run_upgrade(rest),
        "help" => access::run_help(rest),
        "--help" | "-h" => {
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

/// Сбой отвечает машинно, а не падает молча (работа w-legitimate-failure):
/// в режиме `--format json` паника внутреннего нарушения печатает dacc-error
/// с полем legitimate: false — контракт отличает сбой от законного отказа. В
/// обычном режиме остаётся стандартный вывод паники.
fn install_failure_report() {
    let args: Vec<OsString> = std::env::args_os().collect();
    let json = args
        .windows(2)
        .any(|pair| pair[0] == "--format" && pair[1] == "json");
    if !json {
        return;
    }
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        println!("{}", access::error_json(&info.to_string()));
        default(info);
    }));
}
