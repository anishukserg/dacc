use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::{InquiryOutcome, WorkOrigin};
use std::num::NonZeroU16;

dacc_work::work!("w-squash-events",
    title: NonEmptyStr::new("started/gate/landed — один коммит, а не три"),
    slice: crate::slice::s_compact_ceremony,
    origin: WorkOrigin::Inquiry {
        question: NonEmptyStr::new(
            "Как записать started/gate/landed одним коммитом, не потеряв доказательство на дереве и не нарушив «файл события не изменяется»?"
        ),
        produces: InquiryOutcome::Adr,
        timebox_days: NonZeroU16::new(2).unwrap(),
    },
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "ADR: церемония — один коммит на изменение, доказательство на дереве сохранено."
    ),
);
