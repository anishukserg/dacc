use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-gate-tier-wiring",
    title: NonEmptyStr::new("Подключение ярусов калитки"),
    thrust: crate::thrust::t_compiled_registry,
    outcome: NonEmptyStr::new(
        "Хук pre-commit исполняет ярус коммита (без MSRV и зависимостей), work land исполняет полный ярус на дереве коммита и принимает только вердикт с msrv; структурный вердикт события различает ярусы отсутствием или наличием msrv."
    ),
    specification: crate::rfc::rfc_2026_002,
    max_radius: BlastRadius::Crate,
);
