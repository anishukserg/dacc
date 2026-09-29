use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-equivalents",
    title: NonEmptyStr::new("dacc поставляет эквиваленты локальных типов"),
    thrust: crate::thrust::t_reduce_tax,
    outcome: NonEmptyStr::new(
        "Типы invariant!, review!, proposal! и observation! живут в dacc, чтобы локальные типы nerpa можно было мигрировать и удалить."
    ),
    specification: crate::rfc::rfc_2026_001,
    max_radius: BlastRadius::Crate,
);
