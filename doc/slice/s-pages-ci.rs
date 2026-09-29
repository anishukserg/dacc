use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-pages-ci",
    title: NonEmptyStr::new("Публикация сайта на GitHub Pages из CI"),
    thrust: crate::thrust::t_compiled_registry,
    outcome: NonEmptyStr::new(
        "Сайт публикуется на GitHub Pages из actions/pages.yml: Pages включается через enablement, если она не включена на репозитории."
    ),
    specification: crate::rfc::rfc_2026_003,
    max_radius: BlastRadius::Local,
);
