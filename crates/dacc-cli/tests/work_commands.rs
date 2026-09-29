//! Сценарии команд work и slice (решение 15): события пишутся и коммитятся
//! инструментом, незаконный переход и приземление без доказательства
//! отвергаются до записи события.

mod common;

use common::{Run, TempRepo};

/// Репозиторий с таксономией, срезом s-001, работами w-001 и w-002 в нём,
/// каталогом журнала и хуком commit-msg.
fn planned_repo(name: &str) -> TempRepo {
    let repo = TempRepo::new(name);
    repo.write(
        "doc/taxonomy.rs",
        "dacc_core::declare_taxonomy! {\n    Subsystem => [Cli, Work],\n}\n",
    );
    repo.write(
        "doc/slice/s-001.rs",
        "dacc_work::slice!(1,\n    title: NonEmptyStr::new(\"Первый срез\"),\n);\n",
    );
    for (id, title) in [("w-001", "Первая работа"), ("w-002", "Вторая работа")]
    {
        repo.write(
            &format!("doc/work/{id}.rs"),
            &format!(
                "dacc_work::work!(1,\n    title: NonEmptyStr::new(\"{title}\"),\n    slice: crate::slice::s-001,\n    taxon: taxon!(Subsystem, Cli),\n);\n"
            ),
        );
    }
    repo.write("doc/journal/README.md", "журнал\n");
    repo.executable(
        "hooks/commit-msg",
        &format!("#!/bin/sh\nexec '{}' hook commit-msg \"$1\"\n", common::BIN),
    );
    repo.git(&["config", "core.hooksPath", "hooks"]);
    repo.git(&["add", "-A"]);
    repo.git(&[
        "commit",
        "-q",
        "-m",
        "[CHORE](cli): база",
        "-m",
        "Dacc-Work: w-001",
    ]);
    repo
}

/// Минимальный крейт, на котором полный ярус калитки проходит.
fn minimal_crate(repo: &TempRepo) {
    for (path, text) in common::minimal_crate_files() {
        repo.write(path, text);
    }
}

fn state_of(repo: &TempRepo, id: &str) -> String {
    repo.tool(&["work", "state", id]).stdout
}

fn assert_ok(run: &Run) {
    assert_eq!(run.code, 0, "{}", run.output());
    assert!(run.verdict().starts_with("COMMIT OK "), "{}", run.output());
}

#[test]
fn start_land_drop_and_close_write_events_and_commit_them() {
    let repo = planned_repo("work-lifecycle");
    let run = repo.tool(&[
        "work",
        "start",
        "w-001",
        "--trailer",
        "Co-Authored-By: Проба <proba@localhost>",
    ]);
    assert_ok(&run);
    assert!(state_of(&repo, "w-001").contains("started"));
    let body = repo.git(&["log", "-1", "--format=%B"]);
    assert!(
        body.starts_with("[PLAN](cli): work w-001 started"),
        "{body}"
    );
    assert!(
        body.contains("Dacc-Work: w-001\nCo-Authored-By: Проба"),
        "{body}"
    );

    // Без крейта полный ярус калитки не проходит, и приземление отвергается.
    let run = repo.tool(&["work", "land", "w-001"]);
    assert_eq!(run.code, 1, "{}", run.output());
    assert!(run.verdict().contains("full gate tier"), "{}", run.output());

    // Минимальный крейт, на котором полный ярус проходит, — приземление пишет
    // события gate и landed, и вердикт gate несёт msrv полного яруса.
    minimal_crate(&repo);
    repo.git(&["add", "-A"]);
    repo.git(&[
        "commit",
        "-q",
        "-m",
        "[CHORE](cli): минимальный крейт",
        "-m",
        "Dacc-Work: w-001",
    ]);
    let run = repo.tool(&["work", "land", "w-001"]);
    assert_ok(&run);
    let files = repo.git(&["show", "--name-only", "--format=", "HEAD"]);
    assert!(
        files.contains("-gate.toml") && files.contains("-landed.toml"),
        "{files}"
    );
    assert!(state_of(&repo, "w-001").contains("landed"));
    let gate = repo
        .git(&["ls-files", "doc/journal/w-001"])
        .lines()
        .find(|file| file.ends_with("-gate.toml"))
        .expect("событие gate записано")
        .to_owned();
    let event = repo.git(&["show", &format!("HEAD:{gate}")]);
    assert!(event.contains("msrv = \"1.83.0\""), "{event}");

    // Срез не закрывается, пока в нём есть незавершённая работа.
    let run = repo.tool(&["slice", "close", "s-001"]);
    assert_eq!(run.code, 1, "{}", run.output());
    assert!(
        run.verdict().contains("w-002 not finished"),
        "{}",
        run.output()
    );

    assert_ok(&repo.tool(&["work", "drop", "w-002", "--reason", "замещена"]));
    assert!(state_of(&repo, "w-002").contains("abandoned"));
    assert_ok(&repo.tool(&["slice", "close", "s-001"]));
    let body = repo.git(&["log", "-1", "--format=%B"]);
    assert!(body.contains("Dacc-Slice: s-001"), "{body}");
    assert!(repo
        .tool(&["work", "state"])
        .stdout
        .contains("closed slices: s-001"));
}

#[test]
fn illegal_requests_are_refused_before_any_event() {
    let repo = planned_repo("work-illegal");
    let head = repo.git(&["rev-parse", "HEAD"]);
    for (args, code, reason) in [
        (
            vec!["work", "land", "w-001"],
            1,
            "only a started work can be landed",
        ),
        (vec!["work", "start", "w-009"], 1, "is not in the plan"),
        (
            vec!["work", "drop", "w-001", "--reason", " "],
            2,
            "non-empty reason",
        ),
        (vec!["slice", "close", "s-009"], 1, "is not in the plan"),
        (vec!["work", "begin", "w-001"], 2, "work start"),
    ] {
        let run = repo.tool(&args);
        assert_eq!(run.code, code, "{args:?}: {}", run.output());
        assert!(run.verdict().contains(reason), "{args:?}: {}", run.output());
    }
    assert_eq!(
        repo.git(&["rev-parse", "HEAD"]),
        head,
        "отказ создал коммит"
    );

    assert_ok(&repo.tool(&["work", "start", "w-001"]));
    let run = repo.tool(&["work", "start", "w-001"]);
    assert!(
        run.verdict().contains("is already started"),
        "{}",
        run.output()
    );

    // Коммит без трейлера этой работы не приземляет её: основание проверяется до
    // полного яруса.
    repo.write("x.txt", "x\n");
    repo.git(&["add", "x.txt"]);
    repo.git(&[
        "commit",
        "-q",
        "-m",
        "[CHORE](cli): чужая работа",
        "-m",
        "Dacc-Work: w-002",
    ]);
    let run = repo.tool(&["work", "land", "w-001"]);
    assert_eq!(run.code, 1, "{}", run.output());
    assert!(
        run.verdict().contains("is not based on work w-001"),
        "{}",
        run.output()
    );
    assert_eq!(
        repo.git(&["status", "--short", "doc/journal"]),
        "",
        "отказ оставил файлы журнала"
    );
}
