use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!(20,
    title: NonEmptyStr::new("Переименование в DACC"),
    thrust: crate::thrust::t0002,
    outcome: NonEmptyStr::new(
        "Крейты, идентификаторы, бинарь cargo-dacc, настройка dacc.toml и проза носят имя dacc; старые dacc-* на crates.io выведены."
    ),
    specification: crate::rfc::rfc_2026_001,
    max_radius: BlastRadius::Crate,
);
