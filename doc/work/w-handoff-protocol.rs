use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::{InquiryOutcome, WorkOrigin};
use std::num::NonZeroU16;

dacc_work::work!("w-handoff-protocol",
    title: NonEmptyStr::new("Handoff-протокол независимого чтения"),
    slice: crate::slice::s_handoff,
    origin: WorkOrigin::Inquiry {
        question: NonEmptyStr::new(
            "Как устроить handoff независимого чтения вердикта — что читать, в каком порядке, как зовут читателя, что он пишет, — чтобы координация не была ручным шагом каждый раз?"
        ),
        produces: InquiryOutcome::Adr,
        timebox_days: NonZeroU16::new(2).unwrap(),
    },
    taxon: taxon!(Subsystem, Methodology),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "ADR: handoff-протокол независимого чтения зафиксирован."
    ),
);
