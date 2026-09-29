//! JSON всего графа реестра — обмен между инструментами (решение 21).
//! Сериализатор пишет строку вручную, без внешних зависимостей (решение 14).

use dacc_core::{axis, AdrRef, RfcRef, SliceRef, Taxon, WorkRef};
use dacc_knowledge::{ArchitectureDecision, DocStatus, DomainSpecification};
use dacc_work::{Slice, Thrust, WorkItem, WorkOrigin, WorkState};
use std::fmt::Write as _;

/// Версия схемы вывода — часть версионированного контракта.
pub const SCHEMA_VERSION: &str = "1";

/// Экспортирует весь граф реестра одним документом JSON. Идентификаторы
/// документов знания — slug из имени файла, слоя работы — `tNNNN`/`sNNNN`/`wNNNN`.
/// Семь параметров данных — полный снимок реестра; предупреждение об их числе
/// отключается осознанно: дробить снимок ради линта — хуже читаемости.
#[allow(clippy::too_many_arguments)]
pub fn export_graph(
    decisions: &[(&str, &ArchitectureDecision)],
    specs: &[(&str, &DomainSpecification)],
    thrusts: &[&Thrust],
    slices: &[&Slice],
    work: &[&WorkItem],
    states: &[(WorkRef, WorkState)],
    closed: &[SliceRef],
    commit: &str,
) -> String {
    let mut out = String::from("{\n");
    let _ = writeln!(out, "  \"schema\": \"dacc-registry\",");
    let _ = writeln!(out, "  \"version\": \"{}\",", SCHEMA_VERSION);
    let _ = writeln!(out, "  \"commit\": \"{}\",", escape(commit));

    out.push_str("  \"decisions\": [");
    for (i, (id, d)) in decisions.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        let _ = write!(
            out,
            "\n    {{\"id\": \"{}\", \"title\": \"{}\", \"status\": \"{}\", \"subsystems\": [{}], \"related_rfcs\": [{}]}}",
            escape(id),
            escape(d.title),
            status_name(d.status),
            taxa(d.subsystems),
            rfc_refs(d.related_rfcs),
        );
    }
    out.push_str("\n  ],\n");

    out.push_str("  \"specifications\": [");
    for (i, (id, s)) in specs.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        let _ = write!(
            out,
            "\n    {{\"id\": \"{}\", \"title\": \"{}\", \"target\": [{}], \"decided_by\": [{}]}}",
            escape(id),
            escape(s.title),
            taxa(s.target),
            adr_refs(s.decided_by),
        );
    }
    out.push_str("\n  ],\n");

    out.push_str("  \"thrusts\": [");
    for (i, t) in thrusts.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        let _ = write!(
            out,
            "\n    {{\"id\": \"{}\", \"title\": \"{}\"}}",
            escape(t.id),
            escape(t.title.as_str()),
        );
    }
    out.push_str("\n  ],\n");

    out.push_str("  \"slices\": [");
    for (i, s) in slices.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        let _ = write!(
            out,
            "\n    {{\"id\": \"{}\", \"title\": \"{}\", \"thrust\": \"{}\", \"specification\": \"{}\", \"closed\": {}}}",
            escape(s.id),
            escape(s.title.as_str()),
            escape(s.thrust.as_str()),
            s.specification.as_str(),
            is_closed(closed, s.id),
        );
    }
    out.push_str("\n  ],\n");

    out.push_str("  \"work\": [");
    for (i, w) in work.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        let _ = write!(
            out,
            "\n    {{\"id\": \"{}\", \"title\": \"{}\", \"slice\": \"{}\", \"origin\": \"{}\", \"taxon\": \"{}\", \"state\": \"{}\"}}",
            escape(w.id),
            escape(w.title.as_str()),
            escape(w.slice.as_str()),
            escape(&origin_name(&w.origin)),
            escape(w.taxon.as_str()),
            state_name(work_state(states, w.id)),
        );
    }
    out.push_str("\n  ]\n");

    out.push_str("}\n");
    out
}

/// Экранирует строку для строкового литерала JSON.
fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out
}

fn status_name(status: DocStatus) -> &'static str {
    match status {
        DocStatus::Draft => "draft",
        DocStatus::Active => "active",
        DocStatus::Deprecated => "deprecated",
        DocStatus::SupersededBy(_) => "superseded",
    }
}

fn taxa(taxa: &[Taxon<axis::Subsystem>]) -> String {
    taxa.iter()
        .map(|t| format!("\"{}\"", escape(t.as_str())))
        .collect::<Vec<_>>()
        .join(", ")
}

fn rfc_refs(refs: &[RfcRef]) -> String {
    refs.iter()
        .map(|r| format!("\"{}\"", escape(r.as_str())))
        .collect::<Vec<_>>()
        .join(", ")
}

fn adr_refs(refs: &[AdrRef]) -> String {
    refs.iter()
        .map(|r| format!("\"{}\"", escape(r.as_str())))
        .collect::<Vec<_>>()
        .join(", ")
}

fn is_closed(closed: &[SliceRef], id: &str) -> bool {
    closed.iter().any(|c| c.as_str() == id)
}

fn origin_name(origin: &WorkOrigin) -> String {
    match origin {
        WorkOrigin::Decision(r) => format!("decision {}", r.as_str()),
        WorkOrigin::Specification(r) => format!("specification {}", r.as_str()),
        WorkOrigin::Divergence { specification, .. } => {
            format!("divergence {}", specification.as_str())
        }
        WorkOrigin::Inquiry { .. } => "inquiry".to_owned(),
        WorkOrigin::Retirement(r) => format!("retirement {}", r.as_str()),
        WorkOrigin::Toil { .. } => "toil".to_owned(),
    }
}

fn state_name(state: WorkState) -> &'static str {
    match state {
        WorkState::Planned => "planned",
        WorkState::Started => "started",
        WorkState::Landed | WorkState::LandedFromHistory => "landed",
        WorkState::Abandoned => "abandoned",
    }
}

fn work_state(states: &[(WorkRef, WorkState)], id: &str) -> WorkState {
    states
        .iter()
        .find(|(r, _)| r.as_str() == id)
        .map_or(WorkState::Planned, |(_, s)| *s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dacc_core::{nonempty_str, taxon, BlastRadius, NonEmptyStr};
    use dacc_knowledge::{Breaking, DocStatus};

    dacc_core::declare_taxonomy! { Subsystem => [Access] }
    dacc_core::declare_channels! { Public }

    static RELATED_RFCS: &[dacc_core::RfcRef] = &[dacc_core::RfcRef::__from_scan("rfc-2026-003")];
    static DECIDED_BY: &[dacc_core::AdrRef] = &[dacc_core::AdrRef::__from_scan("adr-2026-021")];

    #[test]
    fn escape_covers_json_metacharacters() {
        assert_eq!(escape("a\"b\\c\n\t"), "a\\\"b\\\\c\\n\\t");
    }

    #[test]
    fn graph_exports_nodes_edges_and_states() {
        let decision = ArchitectureDecision {
            title: "Проекция реестра",
            status: DocStatus::Active,
            channel: channel::Public,
            subsystems: Subsystem::ALL,
            context: "Реестр читается только из исходников.",
            decision: "Экспортировать из скомпилированного реестра.",
            trade_offs: &["Плюс: согласованность"],
            constraints: &["Проекция — из констант"],
            authors: nonempty_str!["Анищук Сергей"],
            decided_at: chrono::NaiveDate::from_ymd_opt(2026, 9, 21).unwrap(),
            breaking: Breaking::No,
            code_refs: &[],
            related_rfcs: RELATED_RFCS,
        };
        let spec = DomainSpecification {
            title: "Слой доступа",
            status: DocStatus::Draft,
            channel: channel::Public,
            target: Subsystem::ALL,
            goal: "Интроспекция вместо поиска.",
            input_contract: "Реестры знания и работы.",
            output_contract: "Команды map, state, ...",
            invariants: nonempty_str!["map ≤ 8 КБ"],
            authors: nonempty_str!["Анищук Сергей"],
            decided_at: chrono::NaiveDate::from_ymd_opt(2026, 9, 10).unwrap(),
            decided_by: DECIDED_BY,
        };
        let thrust = Thrust {
            id: "t-003",
            title: NonEmptyStr::new("Спецификация — компилируемый реестр"),
            outcome: NonEmptyStr::new("Норма в реестре"),
        };
        let slice = Slice {
            id: "s-013",
            title: NonEmptyStr::new("Публикуемая проекция"),
            thrust: dacc_core::ThrustRef::__from_scan("t-003"),
            outcome: NonEmptyStr::new("Реестр отдаётся проекцией"),
            specification: dacc_core::RfcRef::__from_scan("rfc-2026-003"),
            max_radius: BlastRadius::Crate,
        };
        let work = WorkItem {
            id: "w-036",
            title: NonEmptyStr::new("Дорожная карта"),
            slice: dacc_core::SliceRef::__from_scan("s-013"),
            origin: dacc_work::WorkOrigin::Decision(dacc_core::AdrRef::__from_scan("adr-2026-021")),
            taxon: taxon!(Subsystem, Access),
            radius: BlastRadius::Local,
            outcome: NonEmptyStr::new("Карта из скомпилированного реестра"),
        };
        let states = [(dacc_core::WorkRef::__from_scan("w-036"), WorkState::Landed)];
        let closed = [dacc_core::SliceRef::__from_scan("s-013")];

        let out = export_graph(
            &[("adr-2026-021", &decision)],
            &[("rfc-2026-003", &spec)],
            &[&thrust],
            &[&slice],
            &[&work],
            &states,
            &closed,
            "abc123",
        );

        assert!(out.contains("\"schema\": \"dacc-registry\""), "{out}");
        assert!(out.contains("\"version\": \"1\""), "{out}");
        assert!(out.contains("\"id\": \"adr-2026-021\""), "{out}");
        assert!(
            out.contains("\"related_rfcs\": [\"rfc-2026-003\"]"),
            "{out}"
        );
        assert!(out.contains("\"thrust\": \"t-003\""), "{out}");
        assert!(out.contains("\"closed\": true"), "{out}");
        assert!(out.contains("\"state\": \"landed\""), "{out}");
    }
}
