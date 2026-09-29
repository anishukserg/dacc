use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-github-polish",
    title: NonEmptyStr::new("Полировка публичной страницы GitHub"),
    thrust: crate::thrust::t_external_pilot,
    outcome: NonEmptyStr::new(
        "README — только суть и ссылки, с плашками версии, калитки, покрытия и мутаций; покрытие считает cargo-llvm-cov → codecov, мутации — cargo-mutants."
    ),
    specification: crate::rfc::rfc_2026_001,
    max_radius: BlastRadius::Local,
);
