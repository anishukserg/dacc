use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-pilot-gate-findings-2",
    title: NonEmptyStr::new("Повторные находки пилота в калитке"),
    thrust: crate::thrust::t_external_pilot,
    outcome: NonEmptyStr::new(
        "Явное дерево калитки уважает .gitignore так же, как рабочее; условие делегирования gate_command вычисляется один раз, а не повторяется в каждом шаге cargo."
    ),
    specification: crate::rfc::rfc_2026_002,
    max_radius: BlastRadius::Crate,
);
