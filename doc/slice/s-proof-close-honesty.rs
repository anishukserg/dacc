use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-proof-close-honesty",
    title: NonEmptyStr::new("Доказательство и закрытие говорят факт: тест с атрибутом, погашенное обязательство"),
    thrust: crate::thrust::t_honest_guarantees,
    outcome: NonEmptyStr::new(
        "RedBefore называет существующий тест дерева — точное имя и атрибут теста; slice close не закрывает срез с непогашенным обязательством и закрывает после погашения работой WorkOrigin::Obligation."
    ),
    specification: crate::rfc::rfc_2026_002,
    max_radius: BlastRadius::Crate,
);
