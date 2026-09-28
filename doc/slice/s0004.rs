use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!(4,
    title: NonEmptyStr::new("Документы DACC в одном крейте doc/"),
    thrust: crate::thrust::t0002,
    outcome: NonEmptyStr::new(
        "Решения, спецификации и план лежат в doc/ одним крейтом dacc-doc, в crates/ только библиотеки; сборка, тесты, калитка и правила коммитов работают по новым путям."
    ),
    specification: crate::rfc::rfc_2026_001,
    max_radius: BlastRadius::Crate,
);
