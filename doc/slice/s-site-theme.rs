use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-site-theme",
    title: NonEmptyStr::new("Тема и вид сайта"),
    thrust: crate::thrust::t_compiled_registry,
    outcome: NonEmptyStr::new(
        "Сайт реестра читается человеком: светлая тема в духе dioxus, схлопываемые второстепенные разделы решений и спецификаций, интерфейс mdBook на русском."
    ),
    specification: crate::rfc::rfc_2026_003,
    max_radius: BlastRadius::Crate,
);
