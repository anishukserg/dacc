use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-publication",
    title: NonEmptyStr::new("Публикация и точка входа раскладки"),
    thrust: crate::thrust::t_external_pilot,
    outcome: NonEmptyStr::new(
        "Крейты знания опубликованы в crates.io и потребляются пилотом по версии; раскладка реестра у потребителя держится библиотечной точкой входа, а не копируемым шаблоном."
    ),
    specification: crate::rfc::rfc_2026_001,
    max_radius: BlastRadius::Crate,
);
