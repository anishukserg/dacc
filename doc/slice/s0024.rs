use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!(24,
    title: NonEmptyStr::new("Slug-идентификаторы слоя работы"),
    thrust: crate::thrust::t0002,
    outcome: NonEmptyStr::new(
        "Направление, срез и работа адресуются slug из имени файла, а не числом; макросы без числового аргумента, ссылки и Subject несут slug, журнал и трейлеры переведены, workspace зелёный."
    ),
    specification: crate::rfc::rfc_2026_002,
    max_radius: BlastRadius::Crate,
);
