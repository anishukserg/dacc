use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-measurable",
    title: NonEmptyStr::new("Замер командой, а не руками"),
    thrust: crate::thrust::t_reduce_tax,
    outcome: NonEmptyStr::new(
        "Шаг внешних имён либо подключён списком, либо честно вычтен из счёта; cargo dacc metrics выводит счётчики пилота одной командой из реестра."
    ),
    specification: crate::rfc::rfc_2026_003,
    max_radius: BlastRadius::Crate,
);
