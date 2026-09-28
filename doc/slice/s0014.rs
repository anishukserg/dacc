use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!(14,
    title: NonEmptyStr::new("Машинный контракт вывода инструмента"),
    thrust: crate::thrust::t0003,
    outcome: NonEmptyStr::new(
        "Потребитель получает вердикт и причины отказа полями, а не предложением: JSON по флагу, стабильные коды причин, структурный вердикт в доказательстве и в событии журнала."
    ),
    specification: crate::rfc::rfc_2026_003,
    max_radius: BlastRadius::Crate,
);
