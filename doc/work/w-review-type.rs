use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::{InquiryOutcome, WorkOrigin};
use std::num::NonZeroU16;

dacc_work::work!("w-review-type",
    title: NonEmptyStr::new("Тип review! в dacc"),
    slice: crate::slice::s_equivalents,
    origin: WorkOrigin::Inquiry {
        question: NonEmptyStr::new(
            "Как выразить независимое чтение (angle, findings, authors, decided_at) типом, чтобы локальный review! nerpa можно было удалить?"
        ),
        produces: InquiryOutcome::Adr,
        timebox_days: NonZeroU16::new(2).unwrap(),
    },
    taxon: taxon!(Subsystem, Knowledge),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "ADR: review! с angle и findings, чтобы локальный тип ушёл."
    ),
);
