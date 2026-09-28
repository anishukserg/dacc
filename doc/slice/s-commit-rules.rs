use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-commit-rules",
    title: NonEmptyStr::new("Правила коммитов и репозиторий без внешних артефактов"),
    thrust: crate::thrust::t_self_work,
    outcome: NonEmptyStr::new(
        "Коммит проходит только с формой сообщения и основанием из плана, калитка гоняется на дереве коммита, строка с именем внешнего проекта отвергается; каталога spec/ в дереве нет."
    ),
    specification: crate::rfc::rfc_2026_002,
    max_radius: BlastRadius::Crate,
);
