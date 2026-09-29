use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-pages-manual",
    title: NonEmptyStr::new("Ручное включение GitHub Pages"),
    thrust: crate::thrust::t_compiled_registry,
    outcome: NonEmptyStr::new(
        "Pages включается вручную в настройках репозитория; enablement: true убран, так как интеграция не имеет права включить Pages."
    ),
    specification: crate::rfc::rfc_2026_003,
    max_radius: BlastRadius::Local,
);
