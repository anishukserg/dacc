use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::{InquiryOutcome, WorkOrigin};
use std::num::NonZeroU16;

dacc_work::work!("w-justified-deviation",
    title: NonEmptyStr::new("Обоснованное отклонение от правила — отдельный тип"),
    slice: crate::slice::s_registry_hygiene,
    origin: WorkOrigin::Inquiry {
        question: NonEmptyStr::new(
            "Как выразить «осознанное отклонение от правила» типом с обязательной причиной и владельцем, отделив его от молчаливого обхода (как contract_allow_justified у AngaraBase)?"
        ),
        produces: InquiryOutcome::Adr,
        timebox_days: NonZeroU16::new(2).unwrap(),
    },
    taxon: taxon!(Subsystem, Knowledge),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "ADR: тип обоснованного исключения с обязательной причиной; молчаливое отклонение невыразимо."
    ),
);
