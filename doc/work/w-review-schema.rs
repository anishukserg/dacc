use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-review-schema",
    title: NonEmptyStr::new("Тип review! в слое работы"),
    slice: crate::slice::s_equivalents,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_046),
    taxon: taxon!(Subsystem, Knowledge),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Запись review! с angle, findings, authors и decided_at: схема, макрос, ссылка ReviewRef, скан реестра doc/review/."
    ),
);
