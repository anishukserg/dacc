use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-declare-baseline",
    title: NonEmptyStr::new("Механизм declare_baseline! и ратчет в скане"),
    slice: crate::slice::s_ratchet_mechanism,
    origin: WorkOrigin::Divergence {
        specification: crate::rfc::rfc_2026_001,
        limitation: crate::limitation::l_baseline_not_built,
    },
    taxon: taxon!(Subsystem, Scan),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Скан принимает declare_baseline! — список известных нарушений с именем и размером; нарушение вне baseline или рост замороженного — ошибка сборки с именем нарушения; уменьшение baseline — отдельное решение, и механизм покрыт тестами на рост и на уменьшение."
    ),
);
