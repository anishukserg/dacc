use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!(15,
    title: NonEmptyStr::new("Обязательства и открытые вопросы в реестре"),
    thrust: crate::thrust::t0003,
    outcome: NonEmptyStr::new(
        "Норма без исполнителя и открытый вопрос записываются в реестр, а не прозой: свёртка их видит, и забыть обязательство нельзя молча."
    ),
    specification: crate::rfc::rfc_2026_002,
    max_radius: BlastRadius::Crate,
);
