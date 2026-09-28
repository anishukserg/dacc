use dacc_core::{BlastRadius, NonEmptyStr};

dacc_work::slice!("s-pilot-gate-findings",
    title: NonEmptyStr::new("Находки пилота в калитке"),
    thrust: crate::thrust::t_external_pilot,
    outcome: NonEmptyStr::new(
        "Калитка уважает .gitignore при обходе дерева и не спотыкается о чужие файлы; проект встраивает собственный гейт без двойного прогона пересекающихся шагов."
    ),
    specification: crate::rfc::rfc_2026_002,
    max_radius: BlastRadius::Crate,
);
