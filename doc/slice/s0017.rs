use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!(17,
    title: NonEmptyStr::new("Читаемые идентификаторы документов знания"),
    thrust: crate::thrust::t0003,
    outcome: NonEmptyStr::new(
        "Документ знания адресуется slug'ом из имени файла, а не числом; правило slug общее с якорями и проверяется сканом; слой работы сохраняет числовые идентификаторы."
    ),
    specification: crate::rfc::rfc_2026_001,
    max_radius: BlastRadius::Crate,
);
