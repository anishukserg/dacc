use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-dacc-md",
    title: NonEmptyStr::new("Актуализация DACC.md"),
    thrust: crate::thrust::t_self_work,
    outcome: NonEmptyStr::new(
        "DACC.md объясняет вертикальный срез: направление, срез и единицу работы как вертикальное изменение через все слои, закрываемое по исходу."
    ),
    specification: crate::rfc::rfc_2026_002,
    max_radius: BlastRadius::Local,
);
