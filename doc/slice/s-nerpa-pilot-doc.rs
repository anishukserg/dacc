use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-nerpa-pilot-doc",
    title: NonEmptyStr::new("Фиксация пилота nerpa в DACC.md"),
    thrust: crate::thrust::t_self_work,
    outcome: NonEmptyStr::new(
        "DACC.md записывает реестр «поймано/пропущено» пилота nerpa (волна 3) в «Честные границы» и строку версии 1.4; оставшийся зазор объявлен срезом s-observable-check-limits."
    ),
    specification: crate::rfc::rfc_2026_002,
    max_radius: BlastRadius::Local,
);
