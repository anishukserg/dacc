use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::{InquiryOutcome, WorkOrigin};
use std::num::NonZeroU16;

dacc_work::work!("w-single-journal",
    title: NonEmptyStr::new("Журнал — один append-only файл, а не файл на событие"),
    slice: crate::slice::s_compact_ceremony,
    origin: WorkOrigin::Inquiry {
        question: NonEmptyStr::new(
            "Как устроить журнал одним append-only файлом вместо файла на событие, не потеряв «событие не изменяется» и свёртку при сборке?"
        ),
        produces: InquiryOutcome::Adr,
        timebox_days: NonZeroU16::new(2).unwrap(),
    },
    taxon: taxon!(Subsystem, Work),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "ADR: журнал — один файл, событие неизменно, свёртка читает его."
    ),
);
