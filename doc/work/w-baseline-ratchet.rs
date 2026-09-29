use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::{InquiryOutcome, WorkOrigin};
use std::num::NonZeroU16;

dacc_work::work!("w-baseline-ratchet",
    title: NonEmptyStr::new("Ратчет миграции: baseline известного долга, блокируются только новые"),
    slice: crate::slice::s_guarantee_and_ratchet,
    origin: WorkOrigin::Inquiry {
        question: NonEmptyStr::new(
            "Как ввести новую строгую проверку на реестр с уже накопленным долгом, не вынуждая чинить весь долг разом: baseline известных расхождений, ратчетом блокируются только новые?"
        ),
        produces: InquiryOutcome::Adr,
        timebox_days: NonZeroU16::new(2).unwrap(),
    },
    taxon: taxon!(Subsystem, Scan),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "ADR: механизм declare_baseline! и ратчет в скане — известный дрейф заморожен и назван, новое нарушение — ошибка сборки."
    ),
);
