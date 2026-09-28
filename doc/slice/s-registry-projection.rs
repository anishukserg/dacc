use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-registry-projection",
    title: NonEmptyStr::new("Публикуемая проекция реестра"),
    thrust: crate::thrust::t_compiled_registry,
    outcome: NonEmptyStr::new(
        "Реестр отдаётся проекцией: машинный срез для инструментов и агентов, сайт для людей. Опубликовано только то дерево, которое прошло калитку, и только то, что помечено публичным."
    ),
    specification: crate::rfc::rfc_2026_003,
    max_radius: BlastRadius::Crate,
);
