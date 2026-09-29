use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-facade-single-dependency",
    title: NonEmptyStr::new("Фасад — единственная зависимость для разметки реестра"),
    thrust: crate::thrust::t_compiled_registry,
    outcome: NonEmptyStr::new(
        "Потребитель подключает один крейт dacc для разметки реестра: порождённый код и макросы ссылаются на фасад, а не на крейты L1."
    ),
    specification: crate::rfc::rfc_2026_001,
    max_radius: BlastRadius::Crate,
);
