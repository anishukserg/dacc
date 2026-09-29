use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-obligation-registry",
    title: NonEmptyStr::new("Обязательства в реестре"),
    thrust: crate::thrust::t_compiled_registry,
    outcome: NonEmptyStr::new(
        "Обязательство — отдельная запись obligation! с обязательным условием погашения; погашение — приземлённая работа с происхождением WorkOrigin::Obligation; свёртка показывает непогашенное, калитка отказывает закрыть срез с непогашенным."
    ),
    specification: crate::rfc::rfc_2026_002,
    max_radius: BlastRadius::Crate,
);
