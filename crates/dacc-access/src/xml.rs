//! XML целевого среза реестра — контекст для агента (решение 21). Документы
//! суть проза со структурой, и теги дают границы цитирования. Пишется вручную,
//! без внешних зависимостей (решение 14).

use crate::json::SCHEMA_VERSION;
use dacc_knowledge::{ArchitectureDecision, DocStatus, DomainSpecification};
use std::fmt::Write as _;

/// Экспортирует срез реестра одним документом XML: решения и спецификации,
/// чей заголовок или подсистемы содержат `question` (без учёта регистра).
/// Пустой вопрос даёт пустой срез — ценность даёт срез по вопросу, а не весь
/// дамп (решение 21).
pub fn export_slice(
    decisions: &[(&str, &ArchitectureDecision)],
    specs: &[(&str, &DomainSpecification)],
    question: &str,
    commit: &str,
) -> String {
    let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    let _ = writeln!(
        out,
        "<registry version=\"{}\" commit=\"{}\">",
        SCHEMA_VERSION,
        escape(commit)
    );

    let q = question.to_lowercase();
    if !q.is_empty() {
        for (id, d) in decisions {
            if matches_decision(d, &q) {
                let _ = writeln!(
                    out,
                    "  <decision id=\"{}\" status=\"{}\">",
                    escape(id),
                    status_name(d.status)
                );
                let _ = writeln!(out, "    <title>{}</title>", escape(d.title));
                let _ = writeln!(out, "    <context>{}</context>", escape(d.context));
                let _ = writeln!(out, "    <decision>{}</decision>", escape(d.decision));
                out.push_str("  </decision>\n");
            }
        }
        for (id, s) in specs {
            if matches_spec(s, &q) {
                let _ = writeln!(out, "  <specification id=\"{}\">", escape(id));
                let _ = writeln!(out, "    <title>{}</title>", escape(s.title));
                let _ = writeln!(out, "    <goal>{}</goal>", escape(s.goal));
                let _ = writeln!(
                    out,
                    "    <input_contract>{}</input_contract>",
                    escape(s.input_contract)
                );
                let _ = writeln!(
                    out,
                    "    <output_contract>{}</output_contract>",
                    escape(s.output_contract)
                );
                out.push_str("  </specification>\n");
            }
        }
    }

    out.push_str("</registry>\n");
    out
}

fn matches_decision(d: &ArchitectureDecision, q: &str) -> bool {
    d.title.to_lowercase().contains(q)
        || d.subsystems
            .iter()
            .any(|t| t.as_str().to_lowercase().contains(q))
}

fn matches_spec(s: &DomainSpecification, q: &str) -> bool {
    s.title.to_lowercase().contains(q)
        || s.target
            .iter()
            .any(|t| t.as_str().to_lowercase().contains(q))
}

fn status_name(status: DocStatus) -> &'static str {
    match status {
        DocStatus::Draft => "draft",
        DocStatus::Active => "active",
        DocStatus::Deprecated => "deprecated",
        DocStatus::SupersededBy(_) => "superseded",
    }
}

/// Экранирует строку для текстового содержимого и атрибутов XML.
fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use dacc_core::nonempty_str;
    use dacc_knowledge::{Breaking, DocStatus};

    dacc_core::declare_taxonomy! { Subsystem => [Access] }

    #[test]
    fn escape_covers_xml_metacharacters() {
        assert_eq!(escape("a&b<c>\"d'e"), "a&amp;b&lt;c&gt;&quot;d&apos;e");
    }

    #[test]
    fn slice_filters_by_question_and_carries_prose() {
        let decision = ArchitectureDecision {
            title: "Проекция реестра",
            status: DocStatus::Active,
            subsystems: Subsystem::ALL,
            context: "Реестр читается только из исходников.",
            decision: "Экспортировать из скомпилированного реестра.",
            trade_offs: &[],
            constraints: &[],
            authors: nonempty_str!["Анищук Сергей"],
            decided_at: chrono::NaiveDate::from_ymd_opt(2026, 9, 21).unwrap(),
            breaking: Breaking::No,
            code_refs: &[],
            related_rfcs: &[],
        };
        let spec = DomainSpecification {
            title: "Слой доступа",
            status: DocStatus::Draft,
            target: Subsystem::ALL,
            goal: "Интроспекция вместо поиска.",
            input_contract: "Реестры знания и работы.",
            output_contract: "Команды map, state, ...",
            invariants: nonempty_str!["map ≤ 8 КБ"],
            authors: nonempty_str!["Анищук Сергей"],
            decided_at: chrono::NaiveDate::from_ymd_opt(2026, 9, 10).unwrap(),
            decided_by: &[],
        };

        let out = export_slice(
            &[("adr-2026-021", &decision)],
            &[("rfc-2026-003", &spec)],
            "проекция",
            "abc123",
        );

        assert!(
            out.contains("<registry version=\"1\" commit=\"abc123\">"),
            "{out}"
        );
        assert!(
            out.contains("<decision id=\"adr-2026-021\" status=\"active\">"),
            "{out}"
        );
        assert!(out.contains("<title>Проекция реестра</title>"), "{out}");

        // Спецификация «Слой доступа» не содержит «проекция» — не попадает в срез.
        assert!(!out.contains("<specification"), "{out}");
    }
}
