//! Сценарии импорта истории в журнал (решение 15): работа с коммитом по
//! трейлеру приземляется из истории, работа без коммита остаётся
//! запланированной, основание импорта не импортируется, а срез, все работы
//! которого завершены, закрывается по флагу.

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

#[test]
fn history_lands_traced_works_and_closes_finished_slices() {
    let repo = history_repo("journal-import");
    let first = repo
        .git(&["rev-list", "--max-parents=0", "HEAD"])
        .trim()
        .to_owned();

    let run = repo.tool(&[
        "journal",
        "import",
        "--work",
        "w-003",
        "--close-finished-slices",
    ]);
    assert_eq!(run.code, 0, "{}", run.output());
    assert!(run.verdict().starts_with("COMMIT OK "), "{}", run.output());

    let state = repo.tool(&["work", "state"]).stdout;
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

    let run = repo.tool(&["journal", "import", "--work", "w-003"]);
    assert_eq!(run.code, 1, "{}", run.output());
    assert!(
        run.verdict().contains("nothing to import"),
        "{}",
        run.output()
    );
}
