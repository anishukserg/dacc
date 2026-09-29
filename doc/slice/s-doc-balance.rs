use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-doc-balance",
    title: NonEmptyStr::new("Фиксация архитектурного баланса в DACC.md"),
    thrust: crate::thrust::t_self_work,
    outcome: NonEmptyStr::new(
        "DACC.md получает раздел «Архитектурный баланс» — реальные зазоры отделены от осознанных компромиссов и инвариантов — и строку версии 1.5."
    ),
    specification: crate::rfc::rfc_2026_002,
    max_radius: BlastRadius::Local,
);
