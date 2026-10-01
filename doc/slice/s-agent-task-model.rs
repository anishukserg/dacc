use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-agent-task-model",
    title: NonEmptyStr::new("Задачная модель агентов: допуск и WIP-лимит"),
    thrust: crate::thrust::t_self_work,
    outcome: NonEmptyStr::new(
        "Агент берёт задачу командой cargo dacc work next: допуск — запланированная работа открытого среза в порядке плана, старт пишется событием под межпроцессным локом журнала, и WIP-лимит из настройки wip_limit отказывает переполнению кодом причины, называя занятые работы."
    ),
    specification: crate::rfc::rfc_2026_002,
    max_radius: BlastRadius::Crate,
);
