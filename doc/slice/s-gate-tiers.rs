use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-gate-tiers",
    title: NonEmptyStr::new("Ярусы калитки"),
    thrust: crate::thrust::t_compiled_registry,
    outcome: NonEmptyStr::new(
        "Калитка делится на ярус коммита и полный ярус закрытия работы: дешёвые шаги — на каждом коммите, MSRV и зависимости — при приземлении; приземление требует полного яруса."
    ),
    specification: crate::rfc::rfc_2026_002,
    max_radius: BlastRadius::Crate,
);
