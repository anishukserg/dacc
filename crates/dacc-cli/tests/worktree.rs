//! Сценарии связанной рабочей копии (работа 43). Доказательство калитки и
//! список внешних имён принадлежат репозиторию, а не копии: методология сама
//! предписывает изоляцию сессий в отдельных копиях (решение 4), и проверка,
//! пройденная в одной из них, обязана оставаться доказательством того же
//! дерева для всех.

mod common;

use common::TempRepo;
use std::fs;
use std::path::PathBuf;

/// Репозиторий с реестром, работой w-001, минимальным крейтом для полного
/// яруса и базовым коммитом по её основанию.
fn planned_repo(name: &str) -> TempRepo {
    let repo = TempRepo::new(name);
    repo.write(
        "doc/taxonomy.rs",
        "dacc_core::declare_taxonomy! {\n    Subsystem => [Cli],\n}\n",
    );
    repo.write(
        "doc/work/w-001.rs",
        "dacc_work::work!(1,\n    title: NonEmptyStr::new(\"Работа\"),\n    taxon: taxon!(Subsystem, Cli),\n);\n",
    );
    repo.write("doc/journal/README.md", "журнал\n");
    for (path, text) in common::minimal_crate_files() {
        repo.write(path, text);
    }
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

/// Связанная рабочая копия рядом с деревом репозитория.
fn worktree(repo: &TempRepo, name: &str) -> PathBuf {
    let path = repo.beside(name);
    repo.git(&[
        "worktree",
        "add",
        "-q",
        path.to_str().expect("путь в UTF-8"),
        "-b",
        name,
    ]);
    path
}

#[test]
fn external_names_are_seen_from_a_linked_worktree() {
    let repo = planned_repo("worktree-names");
    fs::write(repo.path(".git/info/dacc-external-names"), "zzvneshniy\n").expect("список записан");
    let tree = worktree(&repo, "wt-names");
    fs::write(tree.join("leak.txt"), "текст с именем ZZVneshniy внутри\n").expect("файл записан");

    // Список лежит в каталоге репозитория: в копии шаг обязан выполниться, а не
    // назваться невыполненным.
    let run = repo.tool_in(&tree, &["gate"]);
    assert!(!run.stdout.contains("step not run"), "{}", run.output());
    assert!(
        run.stdout.contains("external name in file: leak.txt"),
        "{}",
        run.output()
    );
    assert!(
        run.verdict().starts_with("GATE FAIL: external names"),
        "{}",
        run.output()
    );
}

#[test]
fn the_full_gate_lands_work_from_a_linked_worktree() {
    let repo = planned_repo("worktree-land");
    let tree = worktree(&repo, "wt-land");

    let run = repo.tool_in(&tree, &["work", "start", "w-001"]);
    assert_eq!(run.code, 0, "{}", run.output());

    // Полный ярус исполняется в копии, а не в главной рабочей копии, и пишет
    // события started, gate и landed в дерево копии (решение 42).
    let run = repo.tool_in(&tree, &["work", "land", "w-001"]);
    assert_eq!(run.code, 0, "{}", run.output());
    assert!(run.verdict().starts_with("COMMIT OK "), "{}", run.output());
}
