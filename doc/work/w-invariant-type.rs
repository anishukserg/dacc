use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::{InquiryOutcome, WorkOrigin};
use std::num::NonZeroU16;

dacc_work::work!("w-invariant-type",
    title: NonEmptyStr::new("Тип invariant! в dacc"),
    slice: crate::slice::s_equivalents,
    origin: WorkOrigin::Inquiry {
        question: NonEmptyStr::new(
            "Как выразить инвариант с его статусом (Planned/Claimed/Enforced), описанным bypass и атакующими тестами типом, чтобы локальный тип nerpa можно было удалить?"
        ),
        produces: InquiryOutcome::Adr,
        timebox_days: NonZeroU16::new(2).unwrap(),
    },
    taxon: taxon!(Subsystem, Knowledge),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "ADR: invariant! с проверяемым статусом и атаками, чтобы локальный тип ушёл."
    ),
);
