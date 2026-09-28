use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!(22,
    title: NonEmptyStr::new("Версионирование и выпуск"),
    thrust: crate::thrust::t0004,
    outcome: NonEmptyStr::new(
        "Версия крейтов на crates.io привязана к git-тегу vX.Y.Z и коммиту; политика поднятия версии и порядок выпуска зафиксированы решением, и 0.1.1 опубликована по ним."
    ),
    specification: crate::rfc::rfc_2026_001,
    max_radius: BlastRadius::Crate,
);
