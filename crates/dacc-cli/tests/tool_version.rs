//! Сверка версии инструмента с деревом (ADR-2026-048, работа
//! w-tool-version-check): расхождение версии бинарника и dacc-work в
//! Cargo.lock дерева — отказ мутирующих команд и калитки с кодом
//! tool-version-drift, называющим обе версии и шаг исправления; дерево без
//! Cargo.lock сверки не проходит, read-only команды слоя доступа не отказывают.

mod common;

use common::TempRepo;

/// Репозиторий с записью плана и Cargo.lock, закрепляющим dacc-work: локальный
/// крейт dacc-work подключён зависимостью, и lock сводит их одним коммитом.
fn locked_repo(name: &str, dacc_work: &str) -> TempRepo {
    let repo = TempRepo::new(name);
    repo.write(
        "doc/taxonomy.rs",
        "dacc_core::declare_taxonomy! {\n    Subsystem => [Cli, Work],\n}\n",
    );
    repo.write(
        "doc/slice/s-001.rs",
        "dacc_work::slice!(1,\n    title: NonEmptyStr::new(\"Первый срез\"),\n    specification: crate::rfc::rfc_001,\n);\n",
    );
    repo.write(
        "doc/work/w-001.rs",
        "dacc_work::work!(1,\n    title: NonEmptyStr::new(\"Первая работа\"),\n    slice: crate::slice::s-001,\n    taxon: taxon!(Subsystem, Cli),\n);\n",
    );
    repo.write("doc/journal/README.md", "журнал\n");
    for (path, text) in common::minimal_crate_files() {
        repo.write(path, text);
    }
    repo.write(
        "vendor/dacc-work/Cargo.toml",
        &format!("[package]\nname = \"dacc-work\"\nversion = \"{dacc_work}\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n"),
    );
    repo.write("vendor/dacc-work/src/lib.rs", "//! Vendored dacc-work.\n");
    repo.write(
        "Cargo.toml",
        "[package]\nname = \"probe\"\nversion = \"0.1.0\"\nedition = \"2021\"\nrust-version = \"1.83\"\nlicense = \"MIT\"\n\n[lib]\npath = \"src/lib.rs\"\n\n[dependencies]\ndacc-work = { path = \"vendor/dacc-work\" }\n\n[workspace]\n",
    );
    // Lock обязан быть полным: калитка идёт с --locked.
    let output = common::clean("cargo")
        .current_dir(&repo.root)
        .args(["generate-lockfile", "--offline"])
        .output()
        .expect("cargo не запустился");
    assert!(
        output.status.success(),
        "lock не сгенерирован: {}",
        String::from_utf8_lossy(&output.stderr)
    );
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

/// Расхождение версий отказывает мутирующим командам и калитке: код
/// tool-version-drift, обе версии и шаг исправления названы.
#[test]
fn a_version_drift_refuses_the_tool() {
    let repo = locked_repo("version-drift", "0.0.1");

    let run = repo.tool(&["work", "drop", "w-001", "--reason", "снята"]);
    assert_eq!(run.code, 1, "{}", run.output());
    assert!(
        run.verdict().contains("tool-version-drift"),
        "{}",
        run.output()
    );
    assert!(run.output().contains("0.0.1"), "{}", run.output());
    assert!(
        run.output().contains(env!("CARGO_PKG_VERSION")),
        "{}",
        run.output()
    );

    let run = repo.tool(&["gate"]);
    assert_eq!(run.code, 2, "{}", run.output());
    assert!(
        run.verdict().contains("tool-version-drift"),
        "{}",
        run.output()
    );

    // Read-only команды слоя доступа не отказывают: карта места не ломает.
    let run = repo.tool(&["map"]);
    assert_eq!(run.code, 0, "{}", run.output());
}

/// Совпадение версии проходит, а дерево без Cargo.lock сверки не проходит и
/// не отказывает.
#[test]
fn absent_or_matching_lock_does_not_refuse() {
    let repo = locked_repo("version-match", env!("CARGO_PKG_VERSION"));
    let run = repo.tool(&["gate"]);
    assert_eq!(run.code, 0, "{}", run.output());

    std::fs::remove_file(repo.path("Cargo.lock")).unwrap();
    let run = repo.tool(&["work", "drop", "w-001", "--reason", "снята"]);
    assert_eq!(run.code, 0, "{}", run.output());
}
