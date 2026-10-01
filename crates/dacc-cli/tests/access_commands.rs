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
    repo.write("doc/adr/adr-001.rs", "adr!();\n");
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
