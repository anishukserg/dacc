use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-machine-contract",
    title: NonEmptyStr::new("Машинный контракт вывода инструмента"),
    thrust: crate::thrust::t_compiled_registry,
    outcome: NonEmptyStr::new(
        "Потребитель получает вердикт и причины отказа полями, а не предложением: JSON по флагу, стабильные коды причин, структурный вердикт в доказательстве и в событии журнала."
    ),
    specification: crate::rfc::rfc_2026_003,
    max_radius: BlastRadius::Crate,
);
