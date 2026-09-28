use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-versioning",
    title: NonEmptyStr::new("Версионирование и выпуск"),
    thrust: crate::thrust::t_external_pilot,
    outcome: NonEmptyStr::new(
        "Версия крейтов на crates.io привязана к git-тегу vX.Y.Z и коммиту; политика поднятия версии и порядок выпуска зафиксированы решением, и 0.1.1 опубликована по ним."
    ),
    specification: crate::rfc::rfc_2026_001,
    max_radius: BlastRadius::Crate,
);
