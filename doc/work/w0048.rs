use crate::taxonomy::Subsystem;
use slipway_core::{taxon, BlastRadius, NonEmptyStr};
use slipway_work::{InquiryOutcome, WorkOrigin};
use std::num::NonZeroU16;

slipway_work::work!(48,
    title: NonEmptyStr::new("Ярусы калитки и её бюджет"),
    slice: crate::slice::s0002,
    origin: WorkOrigin::Inquiry {
        question: NonEmptyStr::new(
            "Как калитка делится на ярус коммита и ярус закрытия работы, чем доказательство яруса отличается от полного и что мешает приземлить работу по быстрому ярусу, когда бюджет превышен?"
        ),
        produces: InquiryOutcome::Adr,
        timebox_days: NonZeroU16::new(3).unwrap(),
    },
    taxon: taxon!(Subsystem, Methodology),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "ADR: состав каждого яруса, как ярус попадает в доказательство и в событие, чем приземление требует полного яруса и какой замер объявляет превышение бюджета."
    ),
);
