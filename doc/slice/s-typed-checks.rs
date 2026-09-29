use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-typed-checks",
    title: NonEmptyStr::new("Слабые проверки AngaraBase — типизированными проверками DACC"),
    thrust: crate::thrust::t_borrow_from_angarabase,
    outcome: NonEmptyStr::new(
        "Проверки, которые AngaraBase держит скриптом или ревью, выражены в DACC типом: anti-vacuity пол скана, реестр ограничений со сверкой кода, контрактная граница направлений зависимостей, критерии погашения обязательства."
    ),
    specification: crate::rfc::rfc_2026_002,
    max_radius: BlastRadius::Crate,
);
