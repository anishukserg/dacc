//! Сценарии слоя доступа (решение 21, RFC-0003): карта реестра, место в коде
//! и директивы агентов отвечают без обхода дерева и текстового поиска.

mod common;

use common::TempRepo;

/// Реестр с направлением, срезом и работой — минимальный предмет карты.
fn registry_repo(name: &str) -> TempRepo {
    let repo = TempRepo::new(name);
    repo.write(
        "doc/taxonomy.rs",
        "dacc_core::declare_taxonomy! {\n    Subsystem => [Cli, Work],\n}\n",
    );
    repo.write(
        "doc/thrust/t-001.rs",
        "dacc_work::thrust!(\"t-001\",\n    title: NonEmptyStr::new(\"Первое направление\"),\n    outcome: NonEmptyStr::new(\"Исход первого направления.\"),\n);\n",
    );
    repo.write(
        "doc/slice/s-001.rs",
        "dacc_work::slice!(\"s-001\",\n    title: NonEmptyStr::new(\"Первый срез\"),\n    thrust: crate::thrust::t-001,\n    outcome: NonEmptyStr::new(\"Исход первого среза.\"),\n    specification: crate::rfc::rfc-001,\n    max_radius: BlastRadius::Crate,\n);\n",
    );
    repo.write(
        "doc/work/w-001.rs",
        "dacc_work::work!(\"w-001\",\n    title: NonEmptyStr::new(\"Первая работа\"),\n    slice: crate::slice::s-001,\n    origin: WorkOrigin::Specification(crate::rfc::rfc-001),\n    taxon: taxon!(Subsystem, Cli),\n    radius: BlastRadius::Local,\n    outcome: NonEmptyStr::new(\"Исход первой работы.\"),\n);\n",
    );
    repo.write("doc/adr/adr-001.rs", "adr!(status: DocStatus::Active,);\n");
    repo.git(&["add", "-A"]);
    repo.git(&["commit", "-q", "-m", "[CHORE](cli): реестр"]);
    repo
}

/// Карта реестра отвечает текстом и машинным форматом без find и grep.
#[test]
fn map_reports_the_registry_without_searching() {
    let repo = registry_repo("access-map");

    let run = repo.tool(&["map"]);
    assert_eq!(run.code, 0, "{}", run.output());
    assert!(run.stdout.contains("t-001"), "{}", run.output());
    assert!(
        run.stdout.contains("Первое направление"),
        "{}",
        run.output()
    );
    assert!(run.stdout.contains("s-001"), "{}", run.output());
    assert!(run.stdout.contains("w-001"), "{}", run.output());
    assert!(run.stdout.contains("planned"), "{}", run.output());
    assert!(run.stdout.contains("decisions: 1"), "{}", run.output());

    let json = repo.tool(&["map", "--format", "json"]);
    assert_eq!(json.code, 0, "{}", json.output());
    assert!(
        json.stdout.contains("\"schema\": \"dacc-map\""),
        "{}",
        json.output()
    );
    assert!(
        json.stdout.contains("\"id\": \"w-001\""),
        "{}",
        json.output()
    );
    assert!(
        json.stdout.contains("\"state\": \"planned\""),
        "{}",
        json.output()
    );
    assert!(
        json.stdout.contains("\"decisions\": 1"),
        "{}",
        json.output()
    );
}

/// Место в коде одной командой: where называет разметку файла, документы,
/// ссылающиеся на её якоря, и связанные работы; файл без разметки — отдельный
/// ответ, а не пустой успех.
#[test]
fn where_reports_the_place_and_its_documents() {
    let repo = registry_repo("access-where");
    repo.write(
        "src/lib.rs",
        "#[doc_anchor(id = \"plan-ir\")]\npub struct PlanIr;\n",
    );
    repo.write("doc/adr/adr-002.rs", "adr!(); // проза [plan-ir]\n");
    repo.write(
        "doc/work/w-002.rs",
        "dacc_work::work!(\"w-002\",\n    title: NonEmptyStr::new(\"Вторая работа\"),\n    slice: crate::slice::s-001,\n    origin: WorkOrigin::Decision(crate::adr::adr_002),\n    taxon: taxon!(Subsystem, Cli),\n    radius: BlastRadius::Local,\n    outcome: NonEmptyStr::new(\"Исход второй работы.\"),\n);\n",
    );
    repo.git(&["add", "-A"]);
    repo.git(&["commit", "-q", "-m", "[CHORE](cli): якорь и документ"]);

    let run = repo.tool(&["where", "--file", "src/lib.rs"]);
    assert_eq!(run.code, 0, "{}", run.output());
    assert!(run.stdout.contains("plan-ir"), "{}", run.output());
    assert!(run.stdout.contains("adr-002"), "{}", run.output());
    assert!(run.stdout.contains("w-002"), "{}", run.output());
    assert!(run.stdout.contains("planned"), "{}", run.output());

    let json = repo.tool(&["where", "--file", "src/lib.rs", "--format", "json"]);
    assert_eq!(json.code, 0, "{}", json.output());
    assert!(
        json.stdout.contains("\"schema\": \"dacc-where\""),
        "{}",
        json.output()
    );
    assert!(json.stdout.contains("plan-ir"), "{}", json.output());
    assert!(
        json.stdout.contains("\"id\": \"adr-002\""),
        "{}",
        json.output()
    );

    // Файл без разметки — отдельный ответ, а не пустой успех.
    repo.write("src/other.rs", "pub struct Other;\n");
    let bare = repo.tool(&["where", "--file", "src/other.rs"]);
    assert_eq!(bare.code, 0, "{}", bare.output());
    assert!(bare.stdout.contains("no doc_anchor"), "{}", bare.output());
}

/// Директивы агентов пишутся командой, а не руками: emit all пишет AGENTS.md
/// с правилами навигации и ритуалами.
#[test]
fn emit_all_writes_agent_directives() {
    let repo = registry_repo("access-emit");
    let run = repo.tool(&["emit", "all"]);
    assert_eq!(run.code, 0, "{}", run.output());
    let agents = std::fs::read_to_string(repo.path("AGENTS.md")).unwrap();
    assert!(agents.contains("cargo dacc map"), "{agents}");
    assert!(agents.contains("cargo dacc where"), "{agents}");
    assert!(agents.contains("Dacc-Work"), "{agents}");
    assert!(agents.contains("--red-before"), "{agents}");
}

/// Повторный запуск идентичен закоммиченному файлу, и свежесть проверяема:
/// emit all --check отвергает устаревшую директиву.
#[test]
fn emit_all_is_idempotent_and_checks_freshness() {
    let repo = registry_repo("access-emit-check");
    assert_eq!(repo.tool(&["emit", "all"]).code, 0);
    let first = std::fs::read_to_string(repo.path("AGENTS.md")).unwrap();
    assert_eq!(repo.tool(&["emit", "all"]).code, 0);
    let second = std::fs::read_to_string(repo.path("AGENTS.md")).unwrap();
    assert_eq!(first, second, "повторный emit переписал директиву");

    assert_eq!(repo.tool(&["emit", "all", "--check"]).code, 0);
    std::fs::write(repo.path("AGENTS.md"), "устарело\n").unwrap();
    let stale = repo.tool(&["emit", "all", "--check"]);
    assert_eq!(stale.code, 1, "{}", stale.output());
    assert!(stale.verdict().contains("stale"), "{}", stale.output());
}

/// Перечень и запись: ls перечисляет записи со статусом и фильтрует по нему,
/// show отдаёт запись; незнакомое имя — законный отказ с кодом.
#[test]
fn ls_and_show_answer_from_the_registry() {
    let repo = registry_repo("access-ls-show");

    let ls = repo.tool(&["ls", "work"]);
    assert_eq!(ls.code, 0, "{}", ls.output());
    assert!(ls.stdout.contains("w-001"), "{}", ls.output());
    assert!(ls.stdout.contains("planned"), "{}", ls.output());

    let filtered = repo.tool(&["ls", "adr", "--status", "Active"]);
    assert_eq!(filtered.code, 0, "{}", filtered.output());
    assert!(filtered.stdout.contains("adr-001"), "{}", filtered.output());
    let empty = repo.tool(&["ls", "adr", "--status", "Superseded"]);
    assert_eq!(empty.code, 0, "{}", empty.output());
    assert!(empty.stdout.contains("no records"), "{}", empty.output());

    let show = repo.tool(&["show", "w-001"]);
    assert_eq!(show.code, 0, "{}", show.output());
    assert!(show.stdout.contains("Первая работа"), "{}", show.output());

    let json = repo.tool(&["show", "adr-001", "--format", "json"]);
    assert_eq!(json.code, 0, "{}", json.output());
    assert!(
        json.stdout.contains("\"schema\": \"dacc-show\""),
        "{}",
        json.output()
    );
    assert!(json.stdout.contains("adr-001"), "{}", json.output());

    let missing = repo.tool(&["show", "w-999"]);
    assert_eq!(missing.code, 1, "{}", missing.output());
    assert!(
        missing.verdict().contains("not found"),
        "{}",
        missing.output()
    );

    let kind = repo.tool(&["ls", "нет-такого"]);
    assert_eq!(kind.code, 1, "{}", kind.output());
    assert!(kind.verdict().contains("unknown kind"), "{}", kind.output());
}

/// Граф и поиск: refs называет ссылки записи и ссылающиеся на неё записи,
/// find ищет по реестру; отсутствие совпадений — отдельный ответ.
#[test]
fn refs_and_find_answer_from_the_registry() {
    let repo = registry_repo("access-refs-find");
    repo.write("doc/adr/adr-002.rs", "adr!(); // проза [plan-ir]\n");
    repo.write(
        "doc/work/w-002.rs",
        "dacc_work::work!(\"w-002\",\n    title: NonEmptyStr::new(\"Вторая работа\"),\n    slice: crate::slice::s_001,\n    origin: WorkOrigin::Decision(crate::adr::adr_002),\n    taxon: taxon!(Subsystem, Cli),\n    radius: BlastRadius::Local,\n    outcome: NonEmptyStr::new(\"Исход второй работы.\"),\n);\n",
    );
    repo.git(&["add", "-A"]);
    repo.git(&["commit", "-q", "-m", "[CHORE](cli): работа со ссылкой"]);

    // Исходящие ссылки w-002 — adr-002; входящие adr-002 — w-002.
    let outgoing = repo.tool(&["refs", "w-002"]);
    assert_eq!(outgoing.code, 0, "{}", outgoing.output());
    assert!(outgoing.stdout.contains("adr-002"), "{}", outgoing.output());
    let incoming = repo.tool(&["refs", "adr-002"]);
    assert_eq!(incoming.code, 0, "{}", incoming.output());
    assert!(incoming.stdout.contains("w-002"), "{}", incoming.output());

    let json = repo.tool(&["refs", "w-002", "--format", "json"]);
    assert_eq!(json.code, 0, "{}", json.output());
    assert!(
        json.stdout.contains("\"schema\": \"dacc-refs\""),
        "{}",
        json.output()
    );

    let find = repo.tool(&["find", "Первая"]);
    assert_eq!(find.code, 0, "{}", find.output());
    assert!(find.stdout.contains("w-001"), "{}", find.output());
    let none = repo.tool(&["find", "нет-такого-текста"]);
    assert_eq!(none.code, 0, "{}", none.output());
    assert!(none.stdout.contains("no matches"), "{}", none.output());
}

/// Сводка и изменения дня: brief — открытые срезы и работы, state — события
/// дня и открытая работа в потолке 2 КБ с явным усечением.
#[test]
fn brief_and_state_answer_within_their_budget() {
    let repo = registry_repo("access-brief-state");
    let brief = repo.tool(&["brief"]);
    assert_eq!(brief.code, 0, "{}", brief.output());
    assert!(brief.stdout.contains("w-001"), "{}", brief.output());
    assert!(brief.stdout.contains("s-001"), "{}", brief.output());
    let json = repo.tool(&["brief", "--format", "json"]);
    assert_eq!(json.code, 0, "{}", json.output());
    assert!(
        json.stdout.contains("\"schema\": \"dacc-brief\""),
        "{}",
        json.output()
    );

    let state = repo.tool(&["state"]);
    assert_eq!(state.code, 0, "{}", state.output());

    // Потолок 2 КБ и явное усечение на большом реестре.
    for i in 0..80 {
        repo.write(
            &format!("doc/work/w-1{i:02}.rs"),
            &format!(
                "dacc_work::work!(\"w-1{i:02}\",\n    title: NonEmptyStr::new(\"Длинное имя открытой работы номер {i} {}\"),\n    slice: crate::slice::s_001,\n    taxon: taxon!(Subsystem, Cli),\n);\n",
                "х".repeat(40)
            ),
        );
    }
    repo.git(&["add", "-A"]);
    repo.git(&["commit", "-q", "-m", "[CHORE](cli): много работ"]);
    let big = repo.tool(&["state"]);
    assert_eq!(big.code, 0, "{}", big.output());
    assert!(
        big.stdout.len() <= 2048,
        "ответ state длиной {}",
        big.stdout.len()
    );
    assert!(big.stdout.contains("truncated"), "{}", big.output());
}

/// Машинный контракт: help --format json объявляет стоимость и мутирование
/// каждой команды, и законный отказ в json несёт поле legitimate.
#[test]
fn help_declares_cost_and_refusals_are_legitimate() {
    let repo = registry_repo("access-help");
    let help = repo.tool(&["help", "--format", "json"]);
    assert_eq!(help.code, 0, "{}", help.output());
    for command in [
        "map", "where", "ls", "show", "refs", "find", "brief", "state", "emit", "help",
    ] {
        assert!(
            help.stdout.contains(&format!("\"command\": \"{command}\"")),
            "{}",
            help.output()
        );
    }
    assert!(
        help.stdout.contains("\"mutates\": true"),
        "{}",
        help.output()
    );
    assert!(help.stdout.contains("\"cost\""), "{}", help.output());

    // Законный отказ отличается от сбоя полем legitimate.
    let error = repo.tool(&["show", "w-999", "--format", "json"]);
    assert_eq!(error.code, 1, "{}", error.output());
    assert!(
        error.stdout.contains("\"legitimate\": true"),
        "{}",
        error.output()
    );
    assert!(
        error.stdout.contains("record-not-found"),
        "{}",
        error.output()
    );
}

/// Потолки и xml: map не превышает 8 КБ с явным усечением, map и where
/// отвечают xml с версией контракта.
#[test]
fn map_is_capped_and_xml_carries_the_version() {
    let repo = registry_repo("access-limits");
    repo.write(
        "src/lib.rs",
        "#[doc_anchor(id = \"plan-ir\")]\npub struct PlanIr;\n",
    );
    for i in 0..200 {
        repo.write(
            &format!("doc/work/w-2{i:03}.rs"),
            &format!(
                "dacc_work::work!(\"w-2{i:03}\",\n    title: NonEmptyStr::new(\"Длинное имя работы номер {i} {}\"),\n    slice: crate::slice::s_001,\n    taxon: taxon!(Subsystem, Cli),\n);\n",
                "х".repeat(60)
            ),
        );
    }
    repo.git(&["add", "-A"]);
    repo.git(&["commit", "-q", "-m", "[CHORE](cli): большой реестр"]);

    let big = repo.tool(&["map"]);
    assert_eq!(big.code, 0, "{}", big.output());
    assert!(
        big.stdout.len() <= 8192,
        "ответ map длиной {}",
        big.stdout.len()
    );
    assert!(big.stdout.contains("truncated"), "{}", big.output());

    let xml = repo.tool(&["map", "--format", "xml"]);
    assert_eq!(xml.code, 0, "{}", xml.output());
    assert!(xml.stdout.contains("<map"), "{}", xml.output());
    assert!(xml.stdout.contains("version=\"1\""), "{}", xml.output());

    let where_xml = repo.tool(&["where", "--file", "src/lib.rs", "--format", "xml"]);
    assert_eq!(where_xml.code, 0, "{}", where_xml.output());
    assert!(
        where_xml.stdout.contains("version=\"1\""),
        "{}",
        where_xml.output()
    );
    assert!(
        where_xml.stdout.contains("plan-ir"),
        "{}",
        where_xml.output()
    );
}

/// Минимальный проверщик JSON: скобки и литералы сбалансированы — без
/// зависимостей, как и сам инструмент.
fn json_is_well_formed(text: &str) -> bool {
    let mut depth = 0i32;
    let mut in_string = false;
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if in_string {
            match c {
                '\\' => {
                    chars.next();
                }
                '"' => in_string = false,
                _ => {}
            }
            continue;
        }
        match c {
            '"' => in_string = true,
            '{' | '[' => depth += 1,
            '}' | ']' => {
                depth -= 1;
                if depth < 0 {
                    return false;
                }
            }
            _ => {}
        }
    }
    depth == 0 && !in_string
}

/// Усечение карты происходит до сериализации (работа w-cap-before-render):
/// ответ валиден, укладывается в потолок и сохраняет часть записей, а не
/// вырезает массив целиком.
#[test]
fn map_json_truncation_keeps_context_and_validity() {
    let repo = registry_repo("access-cap");
    for i in 0..200 {
        repo.write(
            &format!("doc/work/w-3{i:03}.rs"),
            &format!(
                "dacc_work::work!(\"w-3{i:03}\",\n    title: NonEmptyStr::new(\"Имя с фигурными }} и кавычками \\\" и переводом\nстроки номер {i} {}\"),\n    slice: crate::slice::s_001,\n    taxon: taxon!(Subsystem, Cli),\n);\n",
                "х".repeat(40)
            ),
        );
    }
    repo.git(&["add", "-A"]);
    repo.git(&["commit", "-q", "-m", "[CHORE](cli): большой реестр"]);

    let json = repo.tool(&["map", "--format", "json"]);
    assert_eq!(json.code, 0, "{}", json.output());
    assert!(
        json.stdout.len() <= 8192,
        "ответ map длиной {}",
        json.stdout.len()
    );
    assert!(
        json.stdout.contains("\"truncated\": true"),
        "{}",
        json.output()
    );
    assert!(
        json_is_well_formed(&json.stdout),
        "json повреждён: {}",
        json.output()
    );
    // Часть записей сохраняется — усечение не вырезает массив целиком.
    assert!(json.stdout.contains("\"id\": \"w-"), "{}", json.output());
}
