use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-guarantee-and-ratchet",
    title: NonEmptyStr::new("Наблюдаемые гарантии и ратчет миграции проверок"),
    thrust: crate::thrust::t_borrow_from_angarabase,
    outcome: NonEmptyStr::new(
        "Гарантия с силой «компилятор» — типизированная запись, связывающая обещание с атакующим compile_fail-тестом и позитивным контролем; новая строгая проверка вводится на реестр с долгом через baseline и ратчет (новые нарушения — ошибка сборки)."
    ),
    specification: crate::rfc::rfc_2026_001,
    max_radius: BlastRadius::Crate,
);
