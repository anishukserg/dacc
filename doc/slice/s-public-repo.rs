use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-public-repo",
    title: NonEmptyStr::new("Публичный репозиторий DACC"),
    thrust: crate::thrust::t_self_work,
    outcome: NonEmptyStr::new(
        "Репозиторий готов к публикации на GitHub: лицензия Apache-2.0, README в корне и у каждого публикуемого крейта, полные метаданные публикации крейтов."
    ),
    specification: crate::rfc::rfc_2026_002,
    max_radius: BlastRadius::Local,
);
