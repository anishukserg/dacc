use slipway_core::{BlastRadius, NonEmptyStr};

slipway_work::slice!(14,
    title: NonEmptyStr::new("Машинный контракт вывода инструмента"),
    thrust: crate::thrust::t0003,
    outcome: NonEmptyStr::new(
        "Потребитель получает вердикт и причины отказа полями, а не предложением: JSON по флагу, стабильные коды причин, структурный вердикт в доказательстве и в событии журнала."
    ),
    specification: crate::rfc::r0003,
    max_radius: BlastRadius::Crate,
);
