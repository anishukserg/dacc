//! Сценарии связанной рабочей копии (работа 43). Доказательство калитки и
//! список внешних имён принадлежат репозиторию, а не копии: методология сама
//! предписывает изоляцию сессий в отдельных копиях (решение 4), и проверка,
//! пройденная в одной из них, обязана оставаться доказательством того же
//! дерева для всех.

mod common;

use common::TempRepo;
use std::fs;
use std::path::PathBuf;

/// Репозиторий с реестром, работой w0001 и базовым коммитом по её основанию.
fn planned_repo(name: &str) -> TempRepo {
    let repo = TempRepo::new(name);
    repo.write(
        "doc/taxonomy.rs",
        "dacc_core::declare_taxonomy! {\n    Subsystem => [Cli],\n}\n",
    );
    repo.write(
        "doc/work/w0001.rs",
        "dacc_work::work!(1,\n    title: NonEmptyStr::new(\"Работа\"),\n    taxon: taxon!(Subsystem, Cli),\n);\n",
    );
    repo.write("doc/journal/README.md", "журнал\n");
    repo.git(&["add", "-A"]);
    repo.git(&[
        "commit",
        "-q",
        "-m",
        "[CHORE](cli): база",
        "-m",
        "Dacc-Work: w0001",
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
fn a_proof_of_the_repository_lands_work_from_a_linked_worktree() {
    let repo = planned_repo("worktree-proof");
    let code = repo.git(&["rev-parse", "HEAD"]).trim().to_owned();
    let tree = worktree(&repo, "wt-proof");

    let run = repo.tool_in(&tree, &["work", "start", "w0001"]);
    assert_eq!(run.code, 0, "{}", run.output());

    // Доказательство принадлежит репозиторию, а не копии, в которой прошла
    // калитка.
    let hash = repo
        .tool_in(&tree, &["journal", "hash", &code])
        .stdout
        .trim()
        .to_owned();
    let proofs = repo.path(".git/dacc-proofs");
    fs::create_dir_all(&proofs).expect("каталог доказательств");
    fs::write(
        proofs.join(&hash),
        "2026-09-21T00:00:00Z\nGATE OK (12 of 12)\n",
    )
    .expect("доказательство записано");

    let run = repo.tool_in(&tree, &["work", "land", "w0001", "--commit", &code]);
    assert_eq!(run.code, 0, "{}", run.output());
    assert!(run.verdict().starts_with("COMMIT OK "), "{}", run.output());
}
