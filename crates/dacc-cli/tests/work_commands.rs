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
    // Решение 42: work start больше не пишет отдельный коммит — событие started
    // появляется в коммите work land.
    let run = repo.tool(&["work", "start", "w-001"]);
    assert_eq!(run.code, 0, "{}", run.output());
    assert!(
        run.stdout
            .contains("work w-001 is planned; it will be recorded when it lands"),
        "{}",
        run.output()
    );
    assert!(state_of(&repo, "w-001").contains("planned"));

    // Без крейта полный ярус калитки не проходит, и приземление отвергается.
    let run = repo.tool(&["work", "land", "w-001"]);
    assert_eq!(run.code, 1, "{}", run.output());
    assert!(run.verdict().contains("full gate tier"), "{}", run.output());

    // Минимальный крейт, на котором полный ярус проходит, — приземление пишет
    // события started, gate и landed одним коммитом, и вердикт gate несёт msrv
    // полного яруса.
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
    // Решение 43: события дописываются в одну запись, а не в отдельные файлы.
    assert!(files.contains("journal.toml"), "{files}");
    assert!(state_of(&repo, "w-001").contains("landed"));
    // Вердикт gate в одной записи несёт msrv полного яруса.
    let journal = repo.git(&["show", "HEAD:doc/journal.toml"]);
    assert!(journal.contains("msrv = \"1.83.0\""), "{journal}");

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

    // Решение 42: work start не пишет коммит и не меняет стадию — повторный
    // start проходит и ничего не пишет.
    assert_eq!(repo.tool(&["work", "start", "w-001"]).code, 0);
    assert_eq!(repo.tool(&["work", "start", "w-001"]).code, 0);
    assert!(state_of(&repo, "w-001").contains("planned"));

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

/// Анти-вакуум (работа w-evidence-land): приземление дефекта — работа с
/// происхождением Divergence — без вида доказательства RedBefore отвергается с
/// кодом причины, а с --red-before событие landed несёт предмет вида.
#[test]
fn land_demands_red_before_for_a_defect_fix() {
    let repo = planned_repo("work-red-before");
    repo.write(
        "doc/work/w-003.rs",
        "dacc_work::work!(1,\n    title: NonEmptyStr::new(\"Починка расхождения\"),\n    slice: crate::slice::s-001,\n    origin: WorkOrigin::Divergence { specification: crate::rfc::rfc-001, limitation: crate::limitation::l-001 },\n    taxon: taxon!(Subsystem, Cli),\n);\n",
    );
    minimal_crate(&repo);
    repo.git(&["add", "-A"]);
    repo.git(&[
        "commit",
        "-q",
        "-m",
        "[FIX](cli): расхождение закрыто",
        "-m",
        "Dacc-Work: w-003",
    ]);

    // Без RedBefore приземление дефекта отвергается до калитки.
    let run = repo.tool(&["work", "land", "w-003"]);
    assert_eq!(run.code, 1, "{}", run.output());
    assert!(run.verdict().contains("--red-before"), "{}", run.output());

    // Вид с предметом приземляется, и событие landed несёт предмет.
    let run = repo.tool(&[
        "work",
        "land",
        "w-003",
        "--red-before",
        "tests::refused_before_the_fix",
    ]);
    assert_ok(&run);
    let journal = repo.git(&["show", "HEAD:doc/journal.toml"]);
    assert!(
        journal.contains("red_before = \"tests::refused_before_the_fix\""),
        "{journal}"
    );
}

/// Пустой предмет вида доказательства — отказ, а не тихо отсутствие вида.
#[test]
fn proof_flags_refuse_an_empty_subject_and_repeats() {
    let repo = planned_repo("work-proof-flags");
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
    for args in [
        vec!["work", "land", "w-001", "--red-before", " "],
        vec![
            "work",
            "land",
            "w-001",
            "--anti-vacuum",
            "x",
            "--anti-vacuum",
            "y",
        ],
    ] {
        let run = repo.tool(&args);
        assert_eq!(run.code, 2, "{args:?}: {}", run.output());
    }
}
