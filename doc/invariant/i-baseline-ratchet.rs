use dacc_core::NonEmptyStr;
use dacc_knowledge::{Enforced, InvariantStatus};

dacc_knowledge::invariant!("i-baseline-ratchet",
    specification: crate::rfc::rfc_2026_001,
    status: InvariantStatus::Enforced(Enforced::Adversarial {
        bypass: NonEmptyStr::new(
            "проверка могла расти поверх baseline, и рост остался бы незамеченным"
        ),
        tests: &[crate::anchor::baseline_slack_is_a_violation],
    }),
    statement: NonEmptyStr::new(
        "Строгая проверка вводится с названным baseline: известный долг заморожен именем и мерой, нарушение вне baseline и любое отклонение меры — ошибка, и baseline обновляется только явным решением (решение 40)."
    ),
    rationale: "Ратчет сверяет текущие нарушения с заморозкой в точности: рост не легализуется молча, уменьшение фиксируется явно, погашенная запись убирается явно. Обход опробован тестом baseline_slack_is_a_violation: уменьшение и рост меры рядом с заморозкой — отказ.",
    enforced_by: &[crate::anchor::baseline_ratchet],
    tests: &[crate::anchor::baseline_slack_is_a_violation],
);
