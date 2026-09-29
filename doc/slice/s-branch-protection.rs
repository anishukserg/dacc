use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-branch-protection",
    title: NonEmptyStr::new("Защита ветки master"),
    thrust: crate::thrust::t_self_work,
    outcome: NonEmptyStr::new(
        "Ветка master защищена правилами GitHub: прямая запись без вердикта CiGate предотвращается, а не только обнаруживается после записи; правила названы и воспроизводимы."
    ),
    specification: crate::rfc::rfc_2026_002,
    max_radius: BlastRadius::Local,
);
