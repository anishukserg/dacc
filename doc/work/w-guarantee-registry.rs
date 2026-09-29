use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::{InquiryOutcome, WorkOrigin};
use std::num::NonZeroU16;

dacc_work::work!("w-guarantee-registry",
    title: NonEmptyStr::new("Гарантия как типизированная запись: обещание + атака + контроль"),
    slice: crate::slice::s_guarantee_and_ratchet,
    origin: WorkOrigin::Inquiry {
        question: NonEmptyStr::new(
            "Как выразить «гарантию с силой компилятор» типом (guarantee!), связывающим обещание с атакующим compile_fail-тестом и позитивным контролем, чтобы висячая ссылка на атаку не компилировалась?"
        ),
        produces: InquiryOutcome::Adr,
        timebox_days: NonZeroU16::new(2).unwrap(),
    },
    taxon: taxon!(Subsystem, Knowledge),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "ADR: тип guarantee! и реестр гарантий, где каждая гарантия ссылается на атаку и контроль; скан порождает константу-ссылку."
    ),
);
