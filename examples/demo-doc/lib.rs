//! Демо-реестр решений. Показывает механизм ссылок-путей.

// Нестабильные возможности запрещены и в doctest: lints манифеста на них не
// распространяются, а атаки выполняются под RUSTC_BOOTSTRAP (решение 12).
#![doc(test(attr(forbid(unstable_features))))]

pub mod taxonomy;

// Порождается build.rs: объявления модулей, константы ссылок, список.
include!(concat!(env!("OUT_DIR"), "/registry.rs"));

// Порождается build.rs: константы разметки кода продукта.
include!(concat!(env!("OUT_DIR"), "/anchors.rs"));

#[cfg(test)]
mod tests {
    use super::*;
    use dacc_knowledge::DocStatus;

    #[test]
    fn registry_is_complete() {
        assert_eq!(ALL.len(), 3);
    }

    #[test]
    fn reference_resolves_to_existing_decision() {
        // adr::adr_direct_plan — путь к константе, порождённой сканом.
        // Опечатка здесь = unresolved path, а не молчаливо неверная ссылка.
        assert_eq!(adr::adr_direct_plan.as_str(), "adr-direct-plan");
        assert_eq!(adr::adr_page_format_v4.as_str(), "adr-page-format-v4");
    }

    #[test]
    fn superseded_module_holds_only_retired_decisions() {
        // Решение «текстовый SQL» замещено — константа есть.
        assert_eq!(superseded::adr_text_sql_path.as_str(), "adr-text-sql-path");
        // Для действующего решения «прямой план» константы в этом модуле НЕТ,
        // поэтому уборка живого кода невыразима:
        //     WorkOrigin::Retirement(superseded::adr_direct_plan)  // unresolved path
    }

    #[test]
    fn work_origin_cannot_point_at_live_decision() {
        // Тип принимает только SupersededRef, а такие константы порождаются
        // лишь для замещённых решений.
        fn retire(_: dacc_core::SupersededRef) {}
        retire(superseded::adr_text_sql_path);
    }

    #[test]
    fn decision_references_real_code() {
        // Решение «прямой план» ссылается на разметку в коде продукта.
        // Удалить #[doc_anchor(id = "plan-ir")] из demo-product — эта строка
        // перестанет резолвиться, потому что константа исчезнет при скане.
        assert_eq!(adr_direct_plan::DECISION.code_refs, &[anchor::plan_ir]);
    }

    #[test]
    fn breaking_decision_carries_migration() {
        let d = &adr_page_format_v4::DECISION;
        match d.breaking {
            dacc_knowledge::Breaking::Yes { migration } => {
                assert_eq!(migration.len(), 2);
            }
            dacc_knowledge::Breaking::No => panic!("ожидалось ломающее решение"),
        }
        assert!(matches!(d.status, DocStatus::Active));
    }
}
