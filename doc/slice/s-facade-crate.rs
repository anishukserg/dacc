use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-facade-crate",
    title: NonEmptyStr::new("Фасадный крейт"),
    thrust: crate::thrust::t_external_pilot,
    outcome: NonEmptyStr::new(
        "Фасадный крейт dacc переэкспортирует публичную поверхность слоёв знания и работы: потребитель подключает одну зависимость вместо набора крейтов."
    ),
    specification: crate::rfc::rfc_2026_001,
    max_radius: BlastRadius::Crate,
);
