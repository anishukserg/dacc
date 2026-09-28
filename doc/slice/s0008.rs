use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!(8,
    title: NonEmptyStr::new("Блокеры пилота L1 на стороннем проекте"),
    thrust: crate::thrust::t0004,
    outcome: NonEmptyStr::new(
        "Крейт документов входит в сторонний проект со строгим профилем lints без исключений; способ доставки крейтов DACC выбран решением; гипотезы, метрики и критерии отказа пилота L1 записаны в реестре до его начала."
    ),
    specification: crate::rfc::rfc_2026_001,
    max_radius: BlastRadius::Crate,
);
