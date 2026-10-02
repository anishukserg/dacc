//! Сценарии импорта истории в журнал (решение 15, работа
//! w-import-attestation): импорт не угадывает — каждое приземление из истории
//! подтверждает человек аргументом `--land <work>` или `--land <work>=<commit>`;
//! без подтверждения команда отказывает и называет кандидатов, а история до
//! трейлеров восстанавливается только названным коммитом.

mod common;

use common::TempRepo;

/// Репозиторий с работами w-001 (коммит по трейлеру), w-002 (без коммита) и
/// w-003 — основанием импорта; срезы s-001 = {w-001}, s-002 = {w-002, w-003}.
fn history_repo(name: &str) -> TempRepo {
    let repo = TempRepo::new(name);
    repo.write(
        "doc/taxonomy.rs",
        "dacc_core::declare_taxonomy! {\n    Subsystem => [Cli, Work],\n}\n",
    );
    for (id, title) in [("s-001", "Первый срез"), ("s-002", "Второй срез")] {
        repo.write(
            &format!("doc/slice/{id}.rs"),
            &format!("dacc_work::slice!(1,\n    title: NonEmptyStr::new(\"{title}\"),\n);\n"),
        );
    }
    for (id, slice) in [("w-001", "s-001"), ("w-002", "s-002"), ("w-003", "s-002")] {
        repo.write(
            &format!("doc/work/{id}.rs"),
            &format!(
                "dacc_work::work!(1,\n    title: NonEmptyStr::new(\n        \"Работа {id}\"\n    ),\n    slice: crate::slice::{slice},\n    taxon: taxon!(Subsystem, Work),\n);\n"
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
        "[FEAT](work): первая работа",
        "-m",
        "Dacc-Work: w-001",
    ]);
    repo.write("code.txt", "код основания\n");
    repo.git(&["add", "code.txt"]);
    repo.git(&[
        "commit",
        "-q",
        "-m",
        "[FEAT](work): импорт",
        "-m",
        "Dacc-Work: w-003",
    ]);
    repo
}

fn state_of(repo: &TempRepo) -> String {
    repo.tool(&["work", "state"]).stdout
}

/// Без `--land` импорт не приземляет ничего: он отказывает и называет
/// кандидатов — работы с коммитами по их трейлерам.
#[test]
fn import_refuses_to_guess_and_names_the_candidates() {
    let repo = history_repo("import-no-guess");

    let run = repo.tool(&["journal", "import", "--work", "w-003"]);
    assert_eq!(run.code, 1, "{}", run.output());
    assert!(run.verdict().contains("does not guess"), "{}", run.output());
    assert!(run.verdict().contains("w-001"), "{}", run.output());
    assert!(!run.verdict().contains("w-003"), "{}", run.output());

    let state = state_of(&repo);
    assert!(state.contains("w-001  planned"), "{state}");
    assert!(state.contains("w-002  planned"), "{state}");
}

/// Подтверждённое приземление записывается по коммиту с трейлером, основание
/// импорта не импортируется, а срез с завершёнными работами закрывается по
/// флагу.
#[test]
fn import_attests_each_landing_explicitly() {
    let repo = history_repo("import-attested");
    let first = repo
        .git(&["rev-list", "--max-parents=0", "HEAD"])
        .trim()
        .to_owned();

    let run = repo.tool(&[
        "journal",
        "import",
        "--work",
        "w-003",
        "--land",
        "w-001",
        "--close-finished-slices",
    ]);
    assert_eq!(run.code, 0, "{}", run.output());
    assert!(run.verdict().starts_with("COMMIT OK "), "{}", run.output());

    let state = state_of(&repo);
    assert!(state.contains("w-001  landed from history"), "{state}");
    assert!(state.contains("w-002  planned"), "{state}");
    assert!(
        state.contains("w-003  planned"),
        "основание импортировано: {state}"
    );
    assert!(state.contains("closed slices: s-001"), "{state}");
    assert!(
        state.contains("Работа w-001"),
        "название с переносом строки не прочитано: {state}"
    );

    let body = repo.git(&["log", "-1", "--format=%B"]);
    assert!(body.contains("Dacc-Work: w-003"), "{body}");
    // Решение 43: приземление дописывается в одну запись, а не в каталог w-001/.
    let journal = repo.git(&["show", "HEAD:doc/journal.toml"]);
    assert!(
        journal.contains(&first),
        "приземление не на коммите с трейлером: {journal}"
    );
    assert!(
        journal.contains("evidence = \"history\""),
        "подтверждённое приземление не помечено историей: {journal}"
    );

    // Второй импорт без подтверждений не находит кандидатов и отказывает.
    let run = repo.tool(&["journal", "import", "--work", "w-003"]);
    assert_eq!(run.code, 1, "{}", run.output());
    assert!(
        run.verdict().contains("nothing to import"),
        "{}",
        run.output()
    );
}

/// История до трейлеров восстанавливается только названным коммитом:
/// `--land <work>=<commit>` приземляет работу на нём, а несуществующий коммит
/// отказывает.
#[test]
fn import_attests_pre_trailer_history_by_named_commit() {
    let repo = history_repo("import-pre-trailer");
    let first = repo
        .git(&["rev-list", "--max-parents=0", "HEAD"])
        .trim()
        .to_owned();

    let run = repo.tool(&[
        "journal",
        "import",
        "--work",
        "w-003",
        "--land",
        &format!("w-002={first}"),
    ]);
    assert_eq!(run.code, 0, "{}", run.output());
    let state = state_of(&repo);
    assert!(state.contains("w-002  landed from history"), "{state}");
    assert!(state.contains("w-001  planned"), "не подтверждено: {state}");

    let run = repo.tool(&[
        "journal",
        "import",
        "--work",
        "w-003",
        "--land",
        "w-001=not-a-commit",
    ]);
    assert_eq!(run.code, 1, "{}", run.output());
    assert!(
        run.verdict().contains("is not a commit"),
        "{}",
        run.output()
    );
}

/// Основание импорта не приземляется своим же импортом, а работа без коммита
/// по трейлеру требует назвать коммит явно.
#[test]
fn import_refuses_the_basis_and_an_untraced_work() {
    let repo = history_repo("import-basis");

    let run = repo.tool(&["journal", "import", "--work", "w-003", "--land", "w-003"]);
    assert_eq!(run.code, 1, "{}", run.output());
    assert!(
        run.verdict().contains("cannot attest its own landing"),
        "{}",
        run.output()
    );

    let run = repo.tool(&["journal", "import", "--work", "w-003", "--land", "w-002"]);
    assert_eq!(run.code, 1, "{}", run.output());
    assert!(run.verdict().contains("w-002=<commit>"), "{}", run.output());
}
