use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-slug-fixes",
    title: NonEmptyStr::new("Исправления slug-перехода"),
    thrust: crate::thrust::t_self_work,
    outcome: NonEmptyStr::new(
        "Команды слоя работы извлекают slug, а не Rust-идентификатор; slice close находит работы среза."
    ),
    specification: crate::rfc::rfc_2026_002,
    max_radius: BlastRadius::Local,
);
