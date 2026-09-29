use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-registry-hygiene",
    title: NonEmptyStr::new("Гигиена реестра и процесса"),
    thrust: crate::thrust::t_borrow_from_angarabase,
    outcome: NonEmptyStr::new(
        "Разделение контракта и проекции состояния, один дом для формы правила, обоснованное отклонение как тип, лейны коммитов."
    ),
    specification: crate::rfc::rfc_2026_003,
    max_radius: BlastRadius::Local,
);
