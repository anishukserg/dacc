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
        "dacc_work::slice!(1,\n    title: NonEmptyStr::new(\"Первый срез\"),\n    specification: crate::rfc::rfc_001,\n);\n",
    );
    repo.write(
        "doc/invariant/i-001.rs",
        "dacc_knowledge::invariant!(\"i-001\",\n    specification: crate::rfc::rfc_001,\n    status: InvariantStatus::Enforced(Enforced::Unrepresentable),\n    statement: NonEmptyStr::new(\"инвариант держится\"),\n    rationale: \"обоснование\",\n    enforced_by: &[crate::anchor::plan_ir],\n    tests: &[crate::anchor::plan_ir],\n);\n",
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

/// Контракт спецификации имеет исполнителя (работа w-contract-close-gate):
/// закрытие среза требует записи инварианта своей спецификации со статусом
/// Enforced и живым якорем enforced_by.
#[test]
fn slice_close_demands_an_enforced_contract() {
    let repo = planned_repo("work-contract-close");
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
    assert_ok(&repo.tool(&["work", "drop", "w-001", "--reason", "снята"]));
    assert_ok(&repo.tool(&["work", "drop", "w-002", "--reason", "снята"]));

    // Спецификация без записи инварианта — контракт без исполнителя.
    std::fs::remove_file(repo.path("doc/invariant/i-001.rs")).unwrap();
    let refused = repo.tool(&["slice", "close", "s-001"]);
    assert_eq!(refused.code, 1, "{}", refused.output());
    assert!(
        refused.verdict().contains("contract"),
        "{}",
        refused.output()
    );

    // Запись без Enforced — тоже отказ.
    repo.write(
        "doc/invariant/i-001.rs",
        "dacc_knowledge::invariant!(\"i-001\",\n    specification: crate::rfc::rfc_001,\n    status: InvariantStatus::Planned,\n    statement: NonEmptyStr::new(\"инвариант\"),\n    rationale: \"обоснование\",\n    enforced_by: &[],\n    tests: &[crate::anchor::plan_ir],\n);\n",
    );
    repo.git(&["add", "-A"]);
    repo.git(&[
        "commit",
        "-q",
        "-m",
        "[CHORE](cli): черновик инварианта",
        "-m",
        "Dacc-Work: w-001",
    ]);
    let refused = repo.tool(&["slice", "close", "s-001"]);
    assert_eq!(refused.code, 1, "{}", refused.output());
    assert!(
        refused.verdict().contains("Enforced"),
        "{}",
        refused.output()
    );

    // Enforced с живым якорем закрывает срез.
    repo.write(
        "doc/invariant/i-001.rs",
        "dacc_knowledge::invariant!(\"i-001\",\n    specification: crate::rfc::rfc_001,\n    status: InvariantStatus::Enforced(Enforced::Unrepresentable),\n    statement: NonEmptyStr::new(\"инвариант держится\"),\n    rationale: \"обоснование\",\n    enforced_by: &[crate::anchor::plan_ir],\n    tests: &[crate::anchor::plan_ir],\n);\n",
    );
    repo.git(&["add", "-A"]);
    repo.git(&[
        "commit",
        "-q",
        "-m",
        "[CHORE](cli): инвариант исполнен",
        "-m",
        "Dacc-Work: w-001",
    ]);
    assert_ok(&repo.tool(&["slice", "close", "s-001"]));
}

/// Разбор значений, а не подстрок (работа w-record-parse): мультилайн,
/// комментарий и перенос строки не обходят проверку контракта.
#[test]
fn slice_close_reads_values_not_substrings() {
    let repo = planned_repo("work-record-parse");
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
    assert_ok(&repo.tool(&["work", "drop", "w-001", "--reason", "снята"]));
    assert_ok(&repo.tool(&["work", "drop", "w-002", "--reason", "снята"]));

    let commit_record = |text: &str, subject: &str| {
        repo.write("doc/invariant/i-001.rs", text);
        repo.git(&["add", "-A"]);
        repo.git(&["commit", "-q", "-m", subject, "-m", "Dacc-Work: w-001"]);
    };

    // Пустой якорь мультилайном — это пустой якорь, а не живой.
    commit_record(
        "dacc_knowledge::invariant!(\"i-001\",\n    specification: crate::rfc::rfc_001,\n    status: InvariantStatus::Enforced(Enforced::Unrepresentable),\n    statement: NonEmptyStr::new(\"инвариант\"),\n    rationale: \"обоснование\",\n    enforced_by: &[\n    ],\n    tests: &[crate::anchor::plan_ir],\n);\n",
        "[CHORE](cli): пустой якорь мультилайном",
    );
    let refused = repo.tool(&["slice", "close", "s-001"]);
    assert_eq!(refused.code, 1, "{}", refused.output());

    // Статус в комментарии и rationale — не статус записи.
    commit_record(
        "dacc_knowledge::invariant!(\"i-001\",\n    specification: crate::rfc::rfc_001,\n    // InvariantStatus::Enforced после рефакторинга\n    status: InvariantStatus::Planned,\n    statement: NonEmptyStr::new(\"инвариант\"),\n    rationale: \"планируется InvariantStatus::Enforced\",\n    enforced_by: &[crate::anchor::plan_ir],\n    tests: &[crate::anchor::plan_ir],\n);\n",
        "[CHORE](cli): статус в комментарии",
    );
    let refused = repo.tool(&["slice", "close", "s-001"]);
    assert_eq!(refused.code, 1, "{}", refused.output());

    // Перенос строки в пути читается верно: валидная запись закрывает срез.
    commit_record(
        "dacc_knowledge::invariant!(\"i-001\",\n    specification:\n        crate::rfc::rfc_001,\n    status: InvariantStatus::Enforced(Enforced::Unrepresentable),\n    statement: NonEmptyStr::new(\"инвариант\"),\n    rationale: \"обоснование\",\n    enforced_by: &[crate::anchor::plan_ir],\n    tests: &[crate::anchor::plan_ir],\n);\n",
        "[CHORE](cli): перенос пути",
    );
    assert_ok(&repo.tool(&["slice", "close", "s-001"]));
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
    repo.write(
        "tests/probe.rs",
        "#[test]\nfn refused_before_the_fix() {}\n",
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
        "refused_before_the_fix",
    ]);
    assert_ok(&run);
    let journal = repo.git(&["show", "HEAD:doc/journal.toml"]);
    assert!(
        journal.contains("red_before = \"refused_before_the_fix\""),
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

/// Запись журнала под межпроцессным локом (работа w-journal-lock):
/// параллельный процесс получает отказ с кодом причины, а не теряет событие.
#[test]
fn journal_write_is_locked_across_processes() {
    let repo = planned_repo("work-journal-lock");
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

    // Лок держит живой процесс — наш собственный pid.
    let lock = repo.path(".git/dacc-journal.lock");
    std::fs::write(&lock, format!("{}", std::process::id())).unwrap();
    let refused = repo.tool(&["work", "drop", "w-001", "--reason", "занято"]);
    assert_eq!(refused.code, 1, "{}", refused.output());
    assert!(refused.verdict().contains("lock"), "{}", refused.output());
    // Событие не потеряно и не записано: работа не тронута.
    assert!(
        state_of(&repo, "w-001").contains("planned"),
        "{}",
        state_of(&repo, "w-001")
    );

    std::fs::remove_file(&lock).unwrap();
    assert_ok(&repo.tool(&["work", "drop", "w-001", "--reason", "свободно"]));
}

/// RedBefore называет существующий тест дерева (работа w-red-before-exists):
/// вымышленное имя отвергается, настоящее приземляет работу.
#[test]
fn red_before_names_a_test_that_exists() {
    let repo = planned_repo("work-red-before-exists");
    repo.write(
        "doc/work/w-003.rs",
        "dacc_work::work!(\"w-003\",\n    title: NonEmptyStr::new(\"Починка расхождения\"),\n    slice: crate::slice::s-001,\n    origin: WorkOrigin::Divergence { specification: crate::rfc::rfc_001, limitation: crate::limitation::l_001 },\n    taxon: taxon!(Subsystem, Cli),\n    radius: BlastRadius::Local,\n    outcome: NonEmptyStr::new(\"Исход починки.\"),\n);\n",
    );
    repo.write("tests/probe.rs", "#[test]\nfn probe_was_red() {}\n");
    minimal_crate(&repo);
    repo.git(&["add", "-A"]);
    repo.git(&[
        "commit",
        "-q",
        "-m",
        "[FIX](cli): починка с тестом",
        "-m",
        "Dacc-Work: w-003",
    ]);

    // Вымышленное имя — отказ: доказательство называет тест дерева.
    let refused = repo.tool(&[
        "work",
        "land",
        "w-003",
        "--red-before",
        "net_takogo_testa_vovse",
    ]);
    assert_eq!(refused.code, 1, "{}", refused.output());
    assert!(refused.verdict().contains("test"), "{}", refused.output());

    // Настоящее имя теста дерева приземляет работу.
    assert_ok(&repo.tool(&["work", "land", "w-003", "--red-before", "probe_was_red"]));
}

/// Enforced требует якоря на исполняемый тест (работа w-enforced-needs-tests):
/// статус без тестового якоря — заявление, а не доказательство.
#[test]
fn slice_close_demands_a_test_anchor_for_enforced() {
    let repo = planned_repo("work-test-anchor");
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
    assert_ok(&repo.tool(&["work", "drop", "w-001", "--reason", "снята"]));
    assert_ok(&repo.tool(&["work", "drop", "w-002", "--reason", "снята"]));

    let commit_record = |text: &str, subject: &str| {
        repo.write("doc/invariant/i-001.rs", text);
        repo.git(&["add", "-A"]);
        repo.git(&["commit", "-q", "-m", subject, "-m", "Dacc-Work: w-001"]);
    };

    // Enforced с живым якорем кода, но без тестового якоря — отказ.
    commit_record(
        "dacc_knowledge::invariant!(\"i-001\",\n    specification: crate::rfc::rfc_001,\n    status: InvariantStatus::Enforced(Enforced::Unrepresentable),\n    statement: NonEmptyStr::new(\"инвариант\"),\n    rationale: \"обоснование\",\n    enforced_by: &[crate::anchor::plan_ir],\n    tests: &[],\n);\n",
        "[CHORE](cli): без тестового якоря",
    );
    let refused = repo.tool(&["slice", "close", "s-001"]);
    assert_eq!(refused.code, 1, "{}", refused.output());
    assert!(
        refused.verdict().contains("test anchor"),
        "{}",
        refused.output()
    );

    // Тестовый якорь закрывает срез.
    commit_record(
        "dacc_knowledge::invariant!(\"i-001\",\n    specification: crate::rfc::rfc_001,\n    status: InvariantStatus::Enforced(Enforced::Unrepresentable),\n    statement: NonEmptyStr::new(\"инвариант\"),\n    rationale: \"обоснование\",\n    enforced_by: &[crate::anchor::plan_ir],\n    tests: &[crate::anchor::probe],\n);\n",
        "[CHORE](cli): тестовый якорь",
    );
    assert_ok(&repo.tool(&["slice", "close", "s-001"]));
}

/// slice close отказывает, пока обязательство работ среза не погашено
/// приземлённой работой WorkOrigin::Obligation (решение 35, работа
/// w-close-obligation): отказ называет обязательство кодом
/// obligation-not-redeemed, а погашение открывает закрытие.
#[test]
fn slice_close_refuses_an_unredeemed_obligation() {
    let repo = planned_repo("work-obligation-close");
    repo.write(
        "doc/obligation/o-001.rs",
        "dacc_work::obligation!(\"o-001\",\n    title: NonEmptyStr::new(\"Погасить открытый вопрос\"),\n    discharged_when: NonEmptyStr::new(\"когда приземлена работа с WorkOrigin::Obligation\"),\n    criteria: nonempty_str![\"названа мера решения\"],\n);\n",
    );
    repo.write(
        "doc/work/w-003.rs",
        "dacc_work::work!(1,\n    title: NonEmptyStr::new(\"Погашение обязательства\"),\n    slice: crate::slice::s-001,\n    origin: WorkOrigin::Obligation(crate::obligation::o_001),\n    taxon: taxon!(Subsystem, Cli),\n);\n",
    );
    minimal_crate(&repo);
    repo.git(&["add", "-A"]);
    repo.git(&[
        "commit",
        "-q",
        "-m",
        "[CHORE](cli): обязательство и крейт",
        "-m",
        "Dacc-Work: w-001",
    ]);
    assert_ok(&repo.tool(&["work", "land", "w-001"]));
    assert_ok(&repo.tool(&["work", "drop", "w-002", "--reason", "замещена"]));
    assert_ok(&repo.tool(&["work", "drop", "w-003", "--reason", "вопрос отложен"]));

    // Обязательство не погашено: закрытие отказывает и называет его.
    let refused = repo.tool(&["slice", "close", "s-001"]);
    assert_eq!(refused.code, 1, "{}", refused.output());
    assert!(refused.verdict().contains("o-001"), "{}", refused.output());
    assert!(
        refused.verdict().contains("obligation-not-redeemed"),
        "{}",
        refused.output()
    );

    // Погашение — приземлённая работа с происхождением WorkOrigin::Obligation.
    repo.write(
        "doc/work/w-004.rs",
        "dacc_work::work!(1,\n    title: NonEmptyStr::new(\"Настоящее погашение\"),\n    slice: crate::slice::s-001,\n    origin: WorkOrigin::Obligation(crate::obligation::o_001),\n    taxon: taxon!(Subsystem, Cli),\n);\n",
    );
    repo.git(&["add", "-A"]);
    repo.git(&[
        "commit",
        "-q",
        "-m",
        "[CHORE](cli): работа погашения",
        "-m",
        "Dacc-Work: w-004",
    ]);
    assert_ok(&repo.tool(&["work", "land", "w-004"]));
    assert_ok(&repo.tool(&["slice", "close", "s-001"]));
}

/// Отказ коммита события откатывает запись журнала целиком: файла нет в
/// дереве и нет в staged-индексе git, и работа остаётся запланированной
/// (работа w-journal-transaction).
#[test]
fn failed_commit_leaves_the_index_clean() {
    let repo = planned_repo("work-rollback");
    // Хук отказывает: коммит события падает после записи файла.
    repo.executable("hooks/commit-msg", "#!/bin/sh\nexit 1\n");

    let run = repo.tool(&["work", "drop", "w-001", "--reason", "не нужна"]);
    assert_ne!(run.code, 0, "{}", run.output());
    assert!(run.stdout.contains("commit-failed"), "{}", run.output());
    let status = repo.git(&["status", "--porcelain"]);
    assert!(
        !status.contains("journal.toml"),
        "след записи в дереве или индексе: {status}"
    );
    assert!(
        state_of(&repo, "w-001").contains("planned"),
        "{}",
        state_of(&repo, "w-001")
    );
}
