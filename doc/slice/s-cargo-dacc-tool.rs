use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-cargo-dacc-tool",
    title: NonEmptyStr::new("Правила коммитов, калитка и хуки — инструмент cargo dacc"),
    thrust: crate::thrust::t_external_pilot,
    outcome: NonEmptyStr::new(
        "Коммиты DACC проходят через cargo dacc commit; хуки — однострочники, вызывающие инструмент, а калитку и pre-push выполняет он; каталога tools/ со скриптами bash нет; сценарии прежнего самотеста — интеграционные тесты крейта dacc-cli."
    ),
    specification: crate::rfc::rfc_2026_002,
    max_radius: BlastRadius::Crate,
);
