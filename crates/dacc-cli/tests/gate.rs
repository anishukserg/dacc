//! Сценарии калитки (решения 8 и 9), перенесённые из самотеста правил
//! коммитов. Все они отказывают до запуска cargo: во временном дереве нет
//! Cargo.toml, и калитка обязана остановиться на этом, а не искать рабочее
//! пространство в родительских каталогах.

mod common;

use common::TempRepo;
use std::fs;

/// Список внешних имён в каталоге git временного репозитория.
fn with_external_names(repo: &TempRepo, names: &str) {
    fs::write(repo.path(".git/info/dacc-external-names"), names).expect("список записан");
}

#[test]
fn external_name_in_the_tree_is_refused() {
    let repo = TempRepo::new("gate-external-name");
    with_external_names(&repo, "# внешние проекты\nzzvneshniy\n");
    repo.write("leak.txt", "текст с именем ZZVneshniy внутри\n");
    let run = repo.tool(&["gate"]);
    assert_eq!(run.code, 1, "{}", run.output());
    assert!(
        run.stdout.contains("  external name in file: leak.txt"),
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
fn ignored_files_are_not_walked_by_the_gate() {
    // Обход рабочего дерева уважает .gitignore: файл в игнорируемом каталоге
    // калитка не видит (работа 57), и она доходит до отсутствия манифеста, а не
    // отказывает на внешнем имени.
    let repo = TempRepo::new("gate-gitignore");
    with_external_names(&repo, "# внешние проекты\nzzvneshniy\n");
    repo.write(".gitignore", ".venv-docs/\n");
    repo.write(".venv-docs/leak.txt", "текст с именем ZZVneshniy внутри\n");
    let run = repo.tool(&["gate"]);
    assert!(
        !run.stdout.contains("external name in file"),
        "{}",
        run.output()
    );
    assert!(
        run.verdict().starts_with("GATE FAIL: no Cargo.toml"),
        "{}",
        run.output()
    );
}

#[test]
fn ignored_files_in_an_explicit_tree_are_not_walked() {
    // Явное дерево — подкаталог репозитория — тоже уважает .gitignore: файл в
    // игнорируемом каталоге калитка не видит, а тот же текст в неигнорируемом
    // файле дерева — видит.
    let repo = TempRepo::new("gate-gitignore-tree");
    with_external_names(&repo, "zzvneshniy\n");
    repo.write(".gitignore", "ignored/\n");
    repo.write(
        "tree/ignored/leak.txt",
        "текст с именем ZZVneshniy внутри\n",
    );
    let root = repo.root.to_str().expect("путь в UTF-8").to_owned();
    let tree = repo.path("tree");
    let tree = tree.to_str().expect("путь в UTF-8");

    let run = repo.tool(&["gate", "--repo", &root, tree]);
    assert!(
        !run.stdout.contains("external name in file"),
        "{}",
        run.output()
    );
    assert!(
        run.verdict()
            .starts_with("GATE FAIL: no Cargo.toml in the tree"),
        "{}",
        run.output()
    );

    // Контроль: то же имя в неигнорируемом файле того же дерева — найдено.
    repo.write("tree/seen.txt", "текст с именем ZZVneshniy внутри\n");
    let run = repo.tool(&["gate", "--repo", &root, tree]);
    assert!(
        run.stdout.contains("external name in file: seen.txt"),
        "{}",
        run.output()
    );
}

#[test]
fn missing_list_is_a_skipped_step_not_a_passed_one() {
    let repo = TempRepo::new("gate-no-list");
    repo.write("leak.txt", "текст с именем ZZVneshniy внутри\n");
    let run = repo.tool(&["gate"]);
    assert!(run.stdout.contains("step not run"), "{}", run.output());
}

#[test]
fn tree_without_manifest_does_not_start_cargo() {
    let repo = TempRepo::new("gate-no-manifest");
    let run = repo.tool(&["gate"]);
    assert_eq!(run.code, 2, "{}", run.output());
    assert!(
        run.verdict()
            .starts_with("GATE FAIL: no Cargo.toml in the tree"),
        "{}",
        run.output()
    );
}

#[test]
fn broken_markdown_link_is_refused_even_inside_a_git_directory() {
    // Дерево коммита выгружается внутрь каталога git: исключение путей по
    // подстроке «/.git/» отсекало бы все файлы, и шаг молча оставался бы без
    // предмета.
    let repo = TempRepo::inside("gate-markdown", ".git");
    repo.write("doc.md", "# Документ\n\nСм. [файл](missing.md).\n");
    let run = repo.tool(&["gate"]);
    assert_eq!(run.code, 1, "{}", run.output());
    assert!(
        run.stdout.contains("  broken link: doc.md: missing.md"),
        "{}",
        run.output()
    );
    assert!(
        run.verdict()
            .starts_with("GATE FAIL: relative links in markdown"),
        "{}",
        run.output()
    );

    // Контроль: ссылка на существующий файл проходит шаг, и калитка доходит до
    // проверки манифеста.
    repo.write("present.md", "есть\n");
    repo.write("doc.md", "# Документ\n\nСм. [файл](present.md#раздел).\n");
    let run = repo.tool(&["gate"]);
    assert!(
        run.verdict()
            .starts_with("GATE FAIL: no Cargo.toml in the tree"),
        "{}",
        run.output()
    );

    // Код в документе — не ссылка: форма темы коммита в обратных кавычках и
    // пример в огороженном блоке кода не отвергают документ (работа 31).
    repo.write(
        "doc.md",
        "# Документ\n\nТема `[ТИП](область): суть`.\n\n```text\n[FEAT](cli): суть\n```\n",
    );
    let run = repo.tool(&["gate"]);
    assert!(!run.stdout.contains("broken link"), "{}", run.output());
    assert!(
        run.verdict()
            .starts_with("GATE FAIL: no Cargo.toml in the tree"),
        "{}",
        run.output()
    );
}

/// Настройка с командой проекта в проверяемом дереве (решение 20).
fn with_gate_command(repo: &TempRepo, command: &str) {
    repo.write("dacc.toml", &format!("gate_command = \"{command}\"\n"));
}

#[test]
fn a_passing_project_command_is_a_step_and_the_gate_goes_on() {
    let repo = TempRepo::new("gate-command-passes");
    with_gate_command(&repo, "true");
    let run = repo.tool(&["gate"]);
    // Шаг выполнен — у него есть журнал, — и калитка дошла до проверки
    // манифеста, как в прочих сценариях.
    assert!(
        repo.path("target/gate/project.log").is_file(),
        "шаг команды проекта не выполнялся: {}",
        run.output()
    );
    assert_eq!(run.code, 2, "{}", run.output());
    assert!(
        run.verdict()
            .starts_with("GATE FAIL: no Cargo.toml in the tree"),
        "{}",
        run.output()
    );
}

#[test]
fn a_failing_project_command_fails_the_gate() {
    let repo = TempRepo::new("gate-command-fails");
    with_gate_command(&repo, "false");
    let run = repo.tool(&["gate"]);
    assert_eq!(run.code, 1, "{}", run.output());
    assert_eq!(
        run.verdict(),
        "GATE FAIL: project command: false (code 1) [step-failed]",
        "{}",
        run.output()
    );
}

#[test]
fn a_missing_project_program_is_refused_not_skipped() {
    let repo = TempRepo::new("gate-command-missing");
    with_gate_command(&repo, "dacc-zzz-no-such-program --check");
    let run = repo.tool(&["gate"]);
    assert_eq!(run.code, 2, "{}", run.output());
    assert!(
        run.verdict().contains("dacc-zzz-no-such-program"),
        "{}",
        run.output()
    );
}

#[test]
fn a_project_command_replaces_fmt_clippy_and_test_but_attacks_still_run() {
    // Решение 28: при заданной команде проекта калитка пропускает стандартные
    // шаги cargo (форматирование, clippy, тесты), но проба сверки кодов и
    // атаки — ядро DACC — по-прежнему выполняются.
    let repo = TempRepo::new("gate-command-replaces-cargo");
    with_gate_command(&repo, "true");
    repo.write(
        "Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\nrust-version = \"1.83\"\n\n[workspace]\n",
    );
    repo.write("src/lib.rs", "//! Demo.\n");
    let run = repo.tool(&["gate"]);

    // Команда проекта выполнена, а fmt/clippy/test пропущены: журналов нет.
    assert!(
        repo.path("target/gate/project.log").is_file(),
        "команда проекта не выполнялась: {}",
        run.output()
    );
    for skipped in ["fmt", "clippy", "test"] {
        assert!(
            !repo.path(&format!("target/gate/{skipped}.log")).is_file(),
            "шаг {skipped} не пропущен при заданной команде проекта: {}",
            run.output()
        );
    }

    // Проба и атаки выполняются: их журналы есть. Калитка отказала на атаках
    // (нет Cargo.lock), а не на пропущенном шаге cargo.
    assert!(
        repo.path("target/gate/probe.log").is_file(),
        "проба сверки кодов не выполнялась: {}",
        run.output()
    );
    assert!(
        repo.path("target/gate/attacks.log").is_file(),
        "атаки не выполнялись: {}",
        run.output()
    );
    assert!(
        run.verdict().starts_with("GATE FAIL: attacks"),
        "{}",
        run.output()
    );
}

#[test]
fn explicit_tree_is_checked_instead_of_the_working_tree() {
    let repo = TempRepo::new("gate-explicit-tree");
    with_external_names(&repo, "zzvneshniy\n");
    repo.write("outside.txt", "ZZVneshniy вне проверяемого дерева\n");
    repo.write("tree/clean.txt", "чисто\n");
    let root = repo.root.to_str().expect("путь в UTF-8").to_owned();
    let tree = repo.path("tree");
    let tree = tree.to_str().expect("путь в UTF-8");

    // Внешнее имя лежит вне дерева: шаг пройден, калитка дошла до манифеста.
    let run = repo.tool(&["gate", "--repo", &root, tree]);
    assert!(
        !run.stdout.contains("external name in file"),
        "{}",
        run.output()
    );
    assert!(
        run.verdict()
            .starts_with("GATE FAIL: no Cargo.toml in the tree"),
        "{}",
        run.output()
    );

    let run = repo.tool(&["gate", tree, "лишний"]);
    assert_eq!(run.code, 2, "{}", run.output());
    assert!(run.verdict().contains("extra argument"), "{}", run.output());
}
