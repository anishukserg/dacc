use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-ratchet-mechanism",
    title: NonEmptyStr::new("Ратчет миграции проверок: declare_baseline! в скане"),
    thrust: crate::thrust::t_borrow_from_angarabase,
    outcome: NonEmptyStr::new(
        "Строгая проверка реестра вводится на реестр с долгом: declare_baseline! объявляет замороженный и названный baseline известных нарушений, скан сравнивает текущее с ним, рост — ошибка сборки, и baseline уменьшается только отдельным решением (решение 40)."
    ),
    specification: crate::rfc::rfc_2026_001,
    max_radius: BlastRadius::Crate,
);
