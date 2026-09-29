use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::{InquiryOutcome, WorkOrigin};
use std::num::NonZeroU16;

dacc_work::work!("w-publication-set",
    title: NonEmptyStr::new("Какой полный набор крейтов публикуется"),
    slice: crate::slice::s_publication_set,
    origin: WorkOrigin::Inquiry {
        question: NonEmptyStr::new(
            "Решение 25 закрепило публикацию пяти крейтов L1; с тех пор выросли слой L2 (dacc-work, dacc-access) и инструмент dacc-cli. Какой полный набор крейтов публикуется и в каком порядке?"
        ),
        produces: InquiryOutcome::Adr,
        timebox_days: NonZeroU16::new(1).unwrap(),
    },
    taxon: taxon!(Subsystem, Methodology),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "ADR: полный набор публикации — пять L1, слой L2 и инструмент cli; порядок по зависимостям; фасадный крейт — отдельно."
    ),
);
