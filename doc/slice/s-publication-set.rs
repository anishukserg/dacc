use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-publication-set",
    title: NonEmptyStr::new("Полный набор публикации"),
    thrust: crate::thrust::t_external_pilot,
    outcome: NonEmptyStr::new(
        "Решением закреплён полный набор публикуемых крейтов — пять L1, слой L2 (work, access) и инструмент cli — и порядок публикации по зависимостям."
    ),
    specification: crate::rfc::rfc_2026_001,
    max_radius: BlastRadius::Crate,
);
