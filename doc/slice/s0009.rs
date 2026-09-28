use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!(9,
    title: NonEmptyStr::new("Публичный репозиторий DACC"),
    thrust: crate::thrust::t0002,
    outcome: NonEmptyStr::new(
        "Репозиторий готов к публикации на GitHub: лицензия Apache-2.0, README в корне и у каждого публикуемого крейта, полные метаданные публикации крейтов."
    ),
    specification: crate::rfc::rfc_2026_002,
    max_radius: BlastRadius::Local,
);
