use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-stable-build",
    title: NonEmptyStr::new("Сборка на stable, атаки на минимальной версии, проверка зависимостей"),
    thrust: crate::thrust::t_honest_guarantees,
    outcome: NonEmptyStr::new(
        "Калитка собирает DACC закреплённым stable, сверяет коды ошибок атак на нём и на минимальной версии и отказывает, если сверка не работает; лицензии, источники и уязвимости зависимостей проверяются cargo-deny."
    ),
    specification: crate::rfc::rfc_2026_002,
    max_radius: BlastRadius::Crate,
);
