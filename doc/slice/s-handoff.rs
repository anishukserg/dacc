use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-handoff",
    title: NonEmptyStr::new("Handoff-протокол независимого чтения"),
    thrust: crate::thrust::t_reduce_tax,
    outcome: NonEmptyStr::new(
        "Порядок независимого чтения вердикта зафиксирован решением: что читать, в каком порядке, как зовут читателя, что он пишет."
    ),
    specification: crate::rfc::rfc_2026_002,
    max_radius: BlastRadius::Local,
);
