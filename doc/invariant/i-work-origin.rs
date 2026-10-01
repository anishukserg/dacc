use dacc_core::NonEmptyStr;
use dacc_knowledge::{Enforced, InvariantStatus};

dacc_knowledge::invariant!("i-work-origin",
    specification: crate::rfc::rfc_2026_002,
    status: InvariantStatus::Enforced(Enforced::Unrepresentable),
    statement: NonEmptyStr::new(
        "Работа класса «реализация решения» ссылается на существующее решение — путём."
    ),
    rationale: "WorkOrigin::Decision несёт AdrRef — путь к константе решения: сослаться на несуществующее решение нечем. Разметка work-origin указывает на тип, в котором это невыразимо.",
    enforced_by: &[crate::anchor::work_origin],
    tests: &[crate::anchor::work_origin_enforced],
);
