use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-facade-crate",
    title: NonEmptyStr::new("Фасадный крейт — публичная поверхность"),
    slice: crate::slice::s_facade_crate,
    origin: WorkOrigin::Divergence {
        specification: crate::rfc::rfc_2026_001,
        limitation: crate::limitation::l_facade_crate,
    },
    taxon: taxon!(Subsystem, Knowledge),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Крейт dacc переэкспортирует публичную поверхность dacc-core, dacc-knowledge, dacc-work, dacc-scan, dacc-access и dacc-journal: типы доступны через один крейт, макросы требуют конкретные крейты из-за $crate."
    ),
);
