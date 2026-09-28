use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-ci-and-contributing",
    title: NonEmptyStr::new("CI и порядок участия в публичном репозитории"),
    thrust: crate::thrust::t_self_work,
    outcome: NonEmptyStr::new(
        "Калитка и проверка оснований коммитов выполняются в GitHub Actions на push и pull request в master; SECURITY.md и CONTRIBUTING.md описывают приватные сообщения об уязвимостях и перенос внешних изменений сопровождающим."
    ),
    specification: crate::rfc::rfc_2026_002,
    max_radius: BlastRadius::Local,
);
