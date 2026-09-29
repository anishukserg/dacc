use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-site-polish",
    title: NonEmptyStr::new("Полировка сайта"),
    thrust: crate::thrust::t_compiled_registry,
    outcome: NonEmptyStr::new(
        "Сайт удобен читателю: боковая панель компактна (короткие названия решений и спецификаций), оглавление читается без горизонтальной прокрутки."
    ),
    specification: crate::rfc::rfc_2026_003,
    max_radius: BlastRadius::Local,
);
