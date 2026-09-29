use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-derived-prose",
    title: NonEmptyStr::new("Выводимое выводится, а не пишется руками"),
    slice: crate::slice::s_typed_prose,
    origin: WorkOrigin::Divergence {
        specification: crate::rfc::rfc_2026_001,
        limitation: crate::limitation::l_derived_prose,
    },
    taxon: taxon!(Subsystem, Scan),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Порядковые номера и статусы порождает скан; смысловая проза остаётся ревью, и её граница названа явно."
    ),
);
