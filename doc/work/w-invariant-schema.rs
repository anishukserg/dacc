use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-invariant-schema",
    title: NonEmptyStr::new("Тип invariant! со статусом в слое работы"),
    slice: crate::slice::s_equivalents,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_045),
    taxon: taxon!(Subsystem, Knowledge),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Запись invariant! со статусом Planned/Claimed/Enforced: схема, макрос, ссылка InvariantRef, скан реестра doc/invariant/."
    ),
);
