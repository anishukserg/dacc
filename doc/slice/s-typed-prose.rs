use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-typed-prose",
    title: NonEmptyStr::new("Выводимое выводится, граница ревью зафиксирована"),
    thrust: crate::thrust::t_reduce_tax,
    outcome: NonEmptyStr::new(
        "Порядковые номера, статусы и счётчики считает скан, а не пишет человек; смысловая проза остаётся ревью, и её граница названа явно."
    ),
    specification: crate::rfc::rfc_2026_001,
    max_radius: BlastRadius::Crate,
);
