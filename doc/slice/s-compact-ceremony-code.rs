use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-compact-ceremony-code",
    title: NonEmptyStr::new("Код: церемония дешевле, журнал одним файлом"),
    thrust: crate::thrust::t_reduce_tax,
    outcome: NonEmptyStr::new(
        "started/gate/landed пишутся одним коммитом командой work land; журнал — одна append-only запись doc/journal.toml, и калитка проверяет, что прежняя версия — префикс новой."
    ),
    specification: crate::rfc::rfc_2026_002,
    max_radius: BlastRadius::Crate,
);
