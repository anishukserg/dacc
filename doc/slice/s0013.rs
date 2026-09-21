use slipway_core::{BlastRadius, NonEmptyStr};

slipway_work::slice!(13,
    title: NonEmptyStr::new("Публикуемая проекция реестра"),
    thrust: crate::thrust::t0003,
    outcome: NonEmptyStr::new(
        "Реестр отдаётся проекцией: машинный срез для инструментов и агентов, сайт для людей. Опубликовано только то дерево, которое прошло калитку, и только то, что помечено публичным."
    ),
    specification: crate::rfc::r0003,
    max_radius: BlastRadius::Crate,
);
