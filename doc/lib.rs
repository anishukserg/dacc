//! Документы самого DACC: решения, спецификации, план работ и журнал.
//!
//! Методология, применённая к себе: решения о том, как устроен DACC, и
//! работа над ним ведутся по правилам DACC. Это первое её испытание — и
//! первое место, где несоблюдение собственных правил стало бы видно
//! немедленно.
//!
//! Все типы документов лежат в одном крейте (решение 11): `adr/`, `rfc/`,
//! `thrust/`, `slice/`, `work/`, `journal/` и таксономия. Внутри крейта
//! документы ссылаются друг на друга путями `crate::`. Знание и план разделяет
//! схема, а не граф крейтов: в решении нет поля для ссылки на работу.
//!
//! Правило плана (решение 5): единицы работы заводятся только на текущие
//! срезы, не больше пяти на срез. Состояния у записей нет: по инварианту 3
//! состояние — свёртка журнала `journal/` при сборке (решение 15), доступная
//! как `WORK_STATES` и `CLOSED_SLICES`.

// Нестабильные возможности запрещены и в doctest: lints манифеста на них не
// распространяются, а атаки выполняются под RUSTC_BOOTSTRAP (решение 12).
#![doc(test(attr(forbid(unstable_features))))]

pub mod taxonomy;

// Порождаются build.rs: модули документов, константы ссылок, списки,
// константные проверки классификации и свёртка журнала.
include!(concat!(env!("OUT_DIR"), "/adr.rs"));
include!(concat!(env!("OUT_DIR"), "/rfc.rs"));
include!(concat!(env!("OUT_DIR"), "/plan.rs"));
include!(concat!(env!("OUT_DIR"), "/journal.rs"));
include!(concat!(env!("OUT_DIR"), "/commit.rs"));

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    /// Непустота авторов и инвариантов обеспечена типом; здесь проверяется
    /// только то, чего тип не выражает: реестры не пусты, у записей есть
    /// заголовки.
    #[test]
    fn registries_are_not_empty() {
        assert!(!ALL.is_empty(), "реестр решений пуст");
        assert!(!ALL_SPECS.is_empty(), "реестр спецификаций пуст");
        for (_, d) in ALL {
            assert!(!d.title.trim().is_empty(), "решение без заголовка");
        }
        for (_, s) in ALL_SPECS {
            assert!(!s.title.trim().is_empty(), "спецификация без заголовка");
        }
    }

    /// Не больше пяти единиц работы на срез. Сила — по требованию: это тест,
    /// а не тип, и держится он тем, что тесты запускают.
    #[test]
    fn work_is_planned_within_wip() {
        for s in ALL_SLICES {
            let n = ALL_WORK.iter().filter(|w| w.slice.as_str() == s.id).count();
            assert!(
                n <= 5,
                "срез «{}» несёт {n} единиц работы",
                s.title.as_str()
            );
        }
    }

    /// Свёртка журнала даёт состояние каждой работе плана.
    #[test]
    fn every_work_has_a_state_from_the_journal() {
        assert_eq!(WORK_STATES.len(), ALL_WORK.len());
    }

    /// Дорожная карта (решение 21) порождается скомпилированным реестром и
    /// несёт коммит сборки; она непуста и разворачивает хотя бы одно
    /// направление.
    #[test]
    fn roadmap_renders_the_plan() {
        let out = dacc_work::roadmap::render_roadmap(
            ALL_THRUSTS,
            ALL_SLICES,
            ALL_WORK,
            WORK_STATES,
            CLOSED_SLICES,
            COMMIT,
        );
        assert!(out.contains("Built from commit"), "{out}");
        assert!(out.contains("## t-honest-guarantees —"), "{out}");
    }

    /// Экспорт (решение 21): JSON графа и XML среза непусты и несут версию.
    #[test]
    fn export_renders_json_and_xml() {
        let json = dacc_access::export_graph(
            ALL,
            ALL_SPECS,
            ALL_THRUSTS,
            ALL_SLICES,
            ALL_WORK,
            WORK_STATES,
            CLOSED_SLICES,
            COMMIT,
        );
        assert!(json.contains("\"schema\": \"dacc-registry\""), "{json}");
        assert!(json.contains("\"id\": \"adr-2026-021\""), "{json}");

        let xml = dacc_access::export_slice(ALL, ALL_SPECS, "проекция", COMMIT);
        assert!(xml.contains("<registry version="), "{xml}");
        assert!(xml.contains("<decision id=\"adr-2026-021\""), "{xml}");
    }

    /// Сайт (решение 21) не расходится с реестром: порождённый из констант
    /// Markdown совпадает с закоммиченным site/src/.
    #[test]
    fn site_is_fresh() {
        let site = dacc_access::render_site(ALL, ALL_SPECS);
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("корень репозитория")
            .join("site/src");
        let summary = fs::read_to_string(root.join("SUMMARY.md")).expect("SUMMARY.md закоммичен");
        assert_eq!(site.summary, summary, "SUMMARY.md разошёлся с реестром");
        for (path, content) in &site.pages {
            let committed = fs::read_to_string(root.join(path))
                .unwrap_or_else(|_| panic!("страница {path} закоммичена"));
            assert_eq!(*content, committed, "страница {path} разошлась с реестром");
        }
    }
}
