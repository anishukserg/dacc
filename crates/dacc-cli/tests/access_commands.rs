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
