use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-toil-radius",
    title: NonEmptyStr::new("Toil ограничен локальным радиусом"),
    thrust: crate::thrust::t_honest_guarantees,
    outcome: NonEmptyStr::new(
        "Работа с происхождением Toil не может объявить радиус выше Local: рутина не трогает публичный API крейта, иначе — ошибка компиляции."
    ),
    specification: crate::rfc::rfc_2026_002,
    max_radius: BlastRadius::Crate,
);
