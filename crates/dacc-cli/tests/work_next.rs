//! Сценарии задачной модели агентов (работа w-work-next): `cargo dacc work
//! next` берёт следующую задачу с допуском, пишет started под локом журнала и
//! держит WIP-лимит из настройки.

mod common;

use common::TempRepo;

/// Репозиторий с таксономией, срезом s-001 и работами w-001 и w-002 в нём.
fn planned_repo(name: &str) -> TempRepo {
    let repo = TempRepo::new(name);
    repo.write(
        "doc/taxonomy.rs",
        "dacc_core::declare_taxonomy! {\n    Subsystem => [Cli, Work],\n}\n",
    );
    repo.write(
        "doc/slice/s-001.rs",
        "dacc_work::slice!(1,\n    title: NonEmptyStr::new(\"Первый срез\"),\n    specification: crate::rfc::rfc_001,\n);\n",
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

/// Допуск берёт следующую запланированную работу открытого среза и пишет
/// событие started в журнал одной записи.
#[test]
fn next_takes_the_next_planned_work_and_writes_started() {
    let repo = planned_repo("next-takes");

    let run = repo.tool(&["work", "next"]);
    assert_eq!(run.code, 0, "{}", run.output());
    assert!(run.stdout.contains("work w-001 taken"), "{}", run.output());
    assert!(run.stdout.contains("wip: 1 of 1"), "{}", run.output());

    let state = repo.tool(&["work", "state", "w-001"]);
    assert!(state.stdout.contains("started"), "{}", state.output());
    let journal = repo.git(&["show", "HEAD:doc/journal.toml"]);
    assert!(journal.contains("event = \"started\""), "{journal}");
    assert!(journal.contains("work = \"w-001\""), "{journal}");
}

/// WIP-лимит отказывает переполнению кодом причины и называет занятую
/// работу: вторая задача не берётся, пока первая в полёте.
#[test]
fn the_wip_limit_refuses_the_second_task_and_names_the_work_in_flight() {
    let repo = planned_repo("next-wip");

    assert_eq!(repo.tool(&["work", "next"]).code, 0);
    let second = repo.tool(&["work", "next"]);
    assert_eq!(second.code, 1, "{}", second.output());
    assert!(second.stdout.contains("wip-limit"), "{}", second.output());
    assert!(second.stdout.contains("w-001"), "{}", second.output());

    // Брошенная работа снимается с полёта, и допуск открывается заново.
    let dropped = repo.tool(&["work", "drop", "w-001", "--reason", "не нужна"]);
    assert_eq!(dropped.code, 0, "{}", dropped.output());
    let third = repo.tool(&["work", "next"]);
    assert_eq!(third.code, 0, "{}", third.output());
    assert!(
        third.stdout.contains("work w-002 taken"),
        "{}",
        third.output()
    );
}

/// WIP-лимит из настройки расширяет допуск до двух задач; третья получает
/// wip-limit, а пустой допуск после завершения обеих — nothing-to-take.
#[test]
fn the_wip_setting_opens_the_admission_and_emptiness_is_refused() {
    let repo = planned_repo("next-setting");
    repo.write("dacc.toml", "wip_limit = \"2\"\n");

    assert_eq!(repo.tool(&["work", "next"]).code, 0);
    assert_eq!(repo.tool(&["work", "next"]).code, 0);
    let third = repo.tool(&["work", "next"]);
    assert_eq!(third.code, 1, "{}", third.output());
    assert!(third.stdout.contains("wip-limit"), "{}", third.output());

    repo.tool(&["work", "drop", "w-001", "--reason", "не нужна"]);
    repo.tool(&["work", "drop", "w-002", "--reason", "не нужна"]);
    let empty = repo.tool(&["work", "next"]);
    assert_eq!(empty.code, 1, "{}", empty.output());
    assert!(
        empty.stdout.contains("nothing-to-take"),
        "{}",
        empty.output()
    );
}

/// Машинный контракт: ответ допуска — json с работой и занятостью лимита.
#[test]
fn next_answers_json_with_the_taken_work() {
    let repo = planned_repo("next-json");

    let run = repo.tool(&["work", "next", "--format", "json"]);
    assert_eq!(run.code, 0, "{}", run.output());
    assert!(
        run.stdout.contains("\"schema\": \"dacc-next\""),
        "{}",
        run.output()
    );
    assert!(run.stdout.contains("\"id\": \"w-001\""), "{}", run.output());
    assert!(
        run.stdout.contains("\"state\": \"started\""),
        "{}",
        run.output()
    );
    assert!(run.stdout.contains("\"taken\": 1"), "{}", run.output());
    assert!(run.stdout.contains("\"limit\": 1"), "{}", run.output());
}

/// Завершённые работы не допускаются: когда брать нечего, отказ отвечает
/// кодом причины nothing-to-take, а не пустым успехом.
#[test]
fn no_admissible_work_is_refused_by_code() {
    let repo = planned_repo("next-empty");
    repo.tool(&["work", "drop", "w-001", "--reason", "не нужна"]);
    repo.tool(&["work", "drop", "w-002", "--reason", "не нужна"]);
    let run = repo.tool(&["work", "next"]);
    assert_eq!(run.code, 1, "{}", run.output());
    assert!(run.stdout.contains("nothing-to-take"), "{}", run.output());
}
