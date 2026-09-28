use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::{InquiryOutcome, WorkOrigin};
use std::num::NonZeroU16;

dacc_work::work!(49,
    title: NonEmptyStr::new("Чем записывается обязательство без исполнителя"),
    slice: crate::slice::s0015,
    origin: WorkOrigin::Inquiry {
        question: NonEmptyStr::new(
            "Как записывается норма без исполнителя и открытый вопрос: отдельной записью реестра с условием погашения, полем существующей записи или долговой книгой, и чем свёртка показывает непогашенное?"
        ),
        produces: InquiryOutcome::Adr,
        timebox_days: NonZeroU16::new(3).unwrap(),
    },
    taxon: taxon!(Subsystem, Work),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "ADR: вид записи обязательства, обязательность условия погашения, место в свёртке и в калитке, и что делает молчаливое забывание обязательства невыразимым."
    ),
);
