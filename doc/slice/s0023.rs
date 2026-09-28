use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!(23,
    title: NonEmptyStr::new("Актуализация публичной документации"),
    thrust: crate::thrust::t0002,
    outcome: NonEmptyStr::new(
        "README отражает опубликованное состояние: крейты в crates.io, dacc-access в таблице крейтов, контракт команд слоя доступа помечен нереализованным."
    ),
    specification: crate::rfc::rfc_2026_001,
    max_radius: BlastRadius::Local,
);
