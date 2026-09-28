use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-rename-to-dacc",
    title: NonEmptyStr::new("Переименование в DACC"),
    thrust: crate::thrust::t_self_work,
    outcome: NonEmptyStr::new(
        "Крейты, идентификаторы, бинарь cargo-dacc, настройка dacc.toml и проза носят имя dacc; старые dacc-* на crates.io выведены."
    ),
    specification: crate::rfc::rfc_2026_001,
    max_radius: BlastRadius::Crate,
);
