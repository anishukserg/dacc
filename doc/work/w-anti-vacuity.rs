use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-anti-vacuity",
    title: NonEmptyStr::new("Anti-vacuity: скан-паттерн краснеет, когда перестал матчиться"),
    slice: crate::slice::s_typed_checks,
    origin: WorkOrigin::Divergence {
        specification: crate::rfc::rfc_2026_002,
        limitation: crate::limitation::l_anti_vacuity,
    },
    taxon: taxon!(Subsystem, Scan),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Проверка нулевой толерантности отличает «паттерн протух» от «чистое дерево»: пол сырых совпадений обязан быть ненулевым, иначе сборка краснеет."
    ),
);
