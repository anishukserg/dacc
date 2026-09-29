use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-compact-ceremony",
    title: NonEmptyStr::new("Церемония дешевле: один коммит на изменение, один журнал"),
    thrust: crate::thrust::t_reduce_tax,
    outcome: NonEmptyStr::new(
        "started/gate/landed пишутся одним коммитом, а не тремя; журнал — один append-only файл, а не файл на событие; доказательство на дереве и неизменность события сохранены."
    ),
    specification: crate::rfc::rfc_2026_002,
    max_radius: BlastRadius::Crate,
);
