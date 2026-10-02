use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-pilot-tool-findings",
    title: NonEmptyStr::new("Находки пилота nerpa об инструменте закрыты исполнителем"),
    thrust: crate::thrust::t_external_pilot,
    outcome: NonEmptyStr::new(
        "Импорт журнала приземляет только подтверждённые работы и коммиты; revert проходит форму темы как отменяемая тема; инструмент отказывает работать вразнос с деревом — каждый пункт доказан сценарием, падавшим до починки."
    ),
    specification: crate::rfc::rfc_2026_002,
    max_radius: BlastRadius::Crate,
);
