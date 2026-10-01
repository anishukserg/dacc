//! Законы форматов журнала (работа w-contract-laws): каждая запись —
//! именованный contract-тест с названным постусловием. Новые зависимости не
//! добавляются: законы покрыты параметрическими прогонами по образцам
//! значений, и набор контролируется cargo-mutants наравне с остальными.

use dacc_journal::{fold, format, Event, Evidence, GateVerdict, Kind, Subject};

const TREE: &str = "0123456789abcdef0123456789abcdef01234567";

/// Постусловие round-trip: разбор вывода возвращает исходные пары, кроме
/// управляющих символов, которые вывод заменяет пробелом.
#[test]
fn law_record_round_trip_keeps_pairs() {
    for sample in [
        "просто",
        "с \"кавычками\"",
        "с \n переводом строки",
        "с \t табом",
        "с \\ слэшем",
        "с иероглифом 漢字",
    ] {
        let fields = vec![
            ("event".to_owned(), "started".to_owned()),
            ("work".to_owned(), sample.to_owned()),
        ];
        let text = format::render(&fields);
        let parsed = format::parse(&text).expect("запись разбирается");
        assert_eq!(parsed.fields, fields, "пары не пережили круг: {sample:?}");
    }
    // Управляющий символ вне перевода строки и табуляции заменяется пробелом —
    // и это часть закона, а не молчаливое усечение.
    let fields = vec![
        ("event".to_owned(), "started".to_owned()),
        ("work".to_owned(), "a\u{1}b".to_owned()),
    ];
    let parsed = format::parse(&format::render(&fields)).expect("запись разбирается");
    assert_eq!(parsed.fields[1].1, "a b");
}

/// Постусловие стабилизации: повторный вывод разобранной записи идентичен
/// исходному каноническому тексту.
#[test]
fn law_render_stabilizes_after_one_pass() {
    let fields = vec![
        ("event".to_owned(), "landed".to_owned()),
        ("work".to_owned(), "w-001".to_owned()),
    ];
    let canonical = format::render(&fields);
    let again = format::render(&format::parse(&canonical).expect("разбор").fields);
    assert_eq!(again, canonical);
}

/// Постусловие монотонности свёртки: приземлённая работа остаётся
/// приземлённой, когда история дописывается законными событиями.
#[test]
fn law_fold_keeps_finished_on_extension() {
    let at = "2026-09-11T03:15:00Z".to_owned();
    let work = Subject::Work("w-001".to_owned());
    let events = vec![
        Event::new(work.clone(), at.clone(), Kind::Started),
        Event::new(
            work.clone(),
            at.clone(),
            Kind::Gate {
                gate: "commit".into(),
                tree: TREE.into(),
                verdict: GateVerdict::Prose("GATE OK".into()),
            },
        ),
        Event::new(
            work,
            at.clone(),
            Kind::Landed {
                commit: TREE.into(),
                tree: TREE.into(),
                evidence: Evidence::Gate,
                proofs: Vec::new(),
            },
        ),
    ];
    let (journal, violations) = fold(&events, &[], &[]);
    assert!(violations.is_empty(), "{violations:?}");
    assert!(journal.stage("w-001").is_finished());

    let mut extended = events;
    extended.push(Event::new(
        Subject::Slice("s-001".to_owned()),
        at,
        Kind::Closed,
    ));
    let (more, _) = fold(&extended, &[], &[]);
    assert!(
        more.stage("w-001").is_finished(),
        "свёртка откатила приземлённое состояние"
    );
}
