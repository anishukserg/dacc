use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-upgrade-guide",
    title: NonEmptyStr::new("Инструкции повышения версии: CHANGELOG, тип upgrade!, команда upgrade"),
    thrust: crate::thrust::t_reduce_tax,
    outcome: NonEmptyStr::new(
        "Потребитель при повышении версии знает, что менять: CHANGELOG.md с upgrade notes, типизированная запись upgrade! (from/to/subject/how), из которой CHANGELOG порождается, и команда cargo dacc upgrade <from> <to>."
    ),
    specification: crate::rfc::rfc_2026_003,
    max_radius: BlastRadius::Crate,
);
