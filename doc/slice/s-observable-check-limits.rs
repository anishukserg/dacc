use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-observable-check-limits",
    title: NonEmptyStr::new("Пределы проверок наблюдаемы"),
    thrust: crate::thrust::t_external_pilot,
    outcome: NonEmptyStr::new(
        "Каждое нарушение называет сущность, файл, строку, сломанную связь и способ исправления; отрицательные сценарии — удалённый якорь, переименование, дубликат ID, устаревшая ссылка, некорректный переход состояния — проверены тестами. Основание: пилот на nerpa показал, что скан ловит переименования и удаления, но его сообщения не называют файл, строку и способ исправления, а семантические расхождения остаются за ревью."
    ),
    specification: crate::rfc::rfc_2026_002,
    max_radius: BlastRadius::Crate,
);
