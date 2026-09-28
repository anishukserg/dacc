use slipway_core::{BlastRadius, NonEmptyStr};

slipway_work::slice!(19,
    title: NonEmptyStr::new("Находки пилота в калитке"),
    thrust: crate::thrust::t0004,
    outcome: NonEmptyStr::new(
        "Калитка уважает .gitignore при обходе дерева и не спотыкается о чужие файлы; проект встраивает собственный гейт без двойного прогона пересекающихся шагов."
    ),
    specification: crate::rfc::rfc_2026_002,
    max_radius: BlastRadius::Crate,
);
