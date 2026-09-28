use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-doc-actualization",
    title: NonEmptyStr::new("Актуализация публичной документации"),
    thrust: crate::thrust::t_self_work,
    outcome: NonEmptyStr::new(
        "README отражает опубликованное состояние: крейты в crates.io, dacc-access в таблице крейтов, контракт команд слоя доступа помечен нереализованным."
    ),
    specification: crate::rfc::rfc_2026_001,
    max_radius: BlastRadius::Local,
);
