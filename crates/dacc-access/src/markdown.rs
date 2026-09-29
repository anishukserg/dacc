//! Markdown-сайт реестра для mdBook (решение 21): страницы из скомпилированного
//! реестра, а не из повторного разбора. Рендерер отдаёт оглавление и страницы;
//! HTML собирает mdBook, публикацию делает CI. План с состояниями — отдельная
//! проекция (дорожная карта), а не часть статичного сайта.

use dacc_knowledge::{ArchitectureDecision, DocStatus, DomainSpecification};
use std::fmt::Write as _;

/// Сайт реестра: оглавление mdBook и страницы Markdown.
pub struct Site {
    /// Содержимое `SUMMARY.md` — навигация книги.
    pub summary: String,
    /// Страницы: путь от корня книги к содержимому Markdown.
    pub pages: Vec<(String, String)>,
}

/// Рендерит сайт реестра: вступление, страницы решений и страницы спецификаций,
/// и оглавление mdBook. Сайт статичен — ни коммита сборки, ни состояний работ,
/// поэтому закоммиченный Markdown совпадает с порождённым из реестра.
pub fn render_site(
    decisions: &[(&str, &ArchitectureDecision)],
    specs: &[(&str, &DomainSpecification)],
) -> Site {
    let mut decisions: Vec<_> = decisions.to_vec();
    decisions.sort_by_key(|(id, _)| *id);
    let mut specs: Vec<_> = specs.to_vec();
    specs.sort_by_key(|(id, _)| *id);

    let mut pages = Vec::new();

    // Вступление.
    pages.push((
        "README.md".to_owned(),
        String::from(
            "# DACC\n\nМетодология, применённая к себе: решения, спецификации, план и \
             журнал записаны в компилируемом реестре и отдаются проекцией. Решения и \
             спецификации — ниже; план с состояниями — дорожная карта.\n",
        ),
    ));

    // Раздел решений.
    pages.push((
        "decisions/README.md".to_owned(),
        String::from("# Decisions\n\nРешения — что и почему решено; список ниже.\n"),
    ));
    for (id, d) in &decisions {
        pages.push((format!("decisions/{id}.md"), render_decision(id, d)));
    }

    // Раздел спецификаций.
    pages.push((
        "specifications/README.md".to_owned(),
        String::from("# Specifications\n\nСпецификации — чего мы хотим; список ниже.\n"),
    ));
    for (id, s) in &specs {
        pages.push((format!("specifications/{id}.md"), render_spec(id, s)));
    }

    // Оглавление mdBook.
    let mut summary = String::from("# Summary\n\n");
    summary.push_str("- [Introduction](README.md)\n");
    summary.push_str("- [Decisions](decisions/README.md)\n");
    for (id, d) in &decisions {
        let _ = writeln!(summary, "  - [{id} — {}](decisions/{id}.md)", d.title);
    }
    summary.push_str("- [Specifications](specifications/README.md)\n");
    for (id, s) in &specs {
        let _ = writeln!(summary, "  - [{id} — {}](specifications/{id}.md)", s.title);
    }

    Site { summary, pages }
}

/// Страница решения: статус, контекст, решение, компромиссы и ограничения.
fn render_decision(id: &str, d: &ArchitectureDecision) -> String {
    let mut out = format!("# {id} — {}\n\n", d.title);
    let _ = writeln!(out, "Status: {}\n", status_name(d.status));
    let _ = writeln!(out, "{}\n", d.context);
    let _ = writeln!(out, "## Decision\n\n{}\n", d.decision);
    if !d.trade_offs.is_empty() {
        out.push_str("## Trade-offs\n\n");
        for t in d.trade_offs {
            let _ = writeln!(out, "- {t}");
        }
        out.push('\n');
    }
    if !d.constraints.is_empty() {
        out.push_str("## Constraints\n\n");
        for c in d.constraints {
            let _ = writeln!(out, "- {c}");
        }
        out.push('\n');
    }
    out
}

/// Страница спецификации: цель, входной и выходной контракты, инварианты.
fn render_spec(id: &str, s: &DomainSpecification) -> String {
    let mut out = format!("# {id} — {}\n\n", s.title);
    let _ = writeln!(out, "{}\n", s.goal);
    let _ = writeln!(out, "## Input contract\n\n{}\n", s.input_contract);
    let _ = writeln!(out, "## Output contract\n\n{}\n", s.output_contract);
    out.push_str("## Invariants\n\n");
    for invariant in s.invariants.iter() {
        let _ = writeln!(out, "- {}", invariant.as_str());
    }
    out.push('\n');
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

#[cfg(test)]
mod tests {
    use super::*;
    use dacc_core::{nonempty_str, taxon};
    use dacc_knowledge::{Breaking, DocStatus};

    dacc_core::declare_taxonomy! { Subsystem => [Access] }

    fn decision(title: &'static str) -> ArchitectureDecision {
        ArchitectureDecision {
            title,
            status: DocStatus::Active,
            subsystems: &[taxon!(Subsystem, Access)],
            context: "Реестр читается только из исходников.",
            decision: "Экспортировать из скомпилированного реестра.",
            trade_offs: &["Плюс: согласованность"],
            constraints: &["Проекция — из констант"],
            authors: nonempty_str!["Анищук Сергей"],
            decided_at: chrono::NaiveDate::from_ymd_opt(2026, 9, 21).unwrap(),
            breaking: Breaking::No,
            code_refs: &[],
            related_rfcs: &[],
        }
    }

    fn spec(title: &'static str) -> DomainSpecification {
        DomainSpecification {
            title,
            status: DocStatus::Draft,
            target: &[taxon!(Subsystem, Access)],
            goal: "Интроспекция вместо поиска.",
            input_contract: "Реестры знания и работы.",
            output_contract: "Команды map, state.",
            invariants: nonempty_str!["map ≤ 8 КБ"],
            authors: nonempty_str!["Анищук Сергей"],
            decided_at: chrono::NaiveDate::from_ymd_opt(2026, 9, 10).unwrap(),
            decided_by: &[],
        }
    }

    #[test]
    fn site_renders_summary_and_pages() {
        let d = decision("Проекция реестра");
        let s = spec("Слой доступа");
        let site = render_site(&[("adr-2026-021", &d)], &[("rfc-2026-003", &s)]);

        assert!(site.summary.contains("- [Introduction](README.md)"));
        assert!(site
            .summary
            .contains("- [adr-2026-021 — Проекция реестра](decisions/adr-2026-021.md)"));

        let readme = page(&site, "README.md");
        assert!(readme.contains("# DACC"));

        let adr = page(&site, "decisions/adr-2026-021.md");
        assert!(adr.contains("# adr-2026-021 — Проекция реестра"));
        assert!(adr.contains("## Decision"));
        assert!(adr.contains("Экспортировать из скомпилированного реестра."));

        let rfc = page(&site, "specifications/rfc-2026-003.md");
        assert!(rfc.contains("## Invariants"));
        assert!(rfc.contains("- map ≤ 8 КБ"));
    }

    fn page<'a>(site: &'a Site, path: &str) -> &'a str {
        site.pages
            .iter()
            .find(|(p, _)| p == path)
            .map(|(_, c)| c.as_str())
            .unwrap_or_else(|| panic!("no page {path}"))
    }
}
