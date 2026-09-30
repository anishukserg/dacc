use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-changelog-generated",
    title: NonEmptyStr::new("Секция Upgrade notes CHANGELOG порождается из реестра"),
    thrust: crate::thrust::t_reduce_tax,
    outcome: NonEmptyStr::new(
        "Секция Upgrade notes CHANGELOG.md — вывод cargo dacc upgrade, а не рукописный текст."
    ),
    specification: crate::rfc::rfc_2026_003,
    max_radius: BlastRadius::Local,
);
