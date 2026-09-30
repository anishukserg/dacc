use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::{InquiryOutcome, WorkOrigin};
use std::num::NonZeroU16;

dacc_work::work!("w-upgrade-type",
    title: NonEmptyStr::new("Тип upgrade!: from/to/subject/how, порождение CHANGELOG и команда upgrade"),
    slice: crate::slice::s_upgrade_guide,
    origin: WorkOrigin::Inquiry {
        question: NonEmptyStr::new(
            "Как оформить инструкцию повышения версии типом upgrade! (from/to/subject/how), порождать из неё CHANGELOG и выводить шаги командой cargo dacc upgrade <from> <to>?"
        ),
        produces: InquiryOutcome::Adr,
        timebox_days: NonZeroU16::new(2).unwrap(),
    },
    taxon: taxon!(Subsystem, Knowledge),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "ADR: upgrade! с полями from/to/subject/how; CHANGELOG порождается из реестра, а не пишется руками; команда upgrade выводит шаги между версиями."
    ),
);
