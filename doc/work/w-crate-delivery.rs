use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::{InquiryOutcome, WorkOrigin};
use std::num::NonZeroU16;

dacc_work::work!("w-crate-delivery",
    title: NonEmptyStr::new("Доставка крейтов DACC в сторонний проект"),
    slice: crate::slice::s_pilot_blockers,
    origin: WorkOrigin::Inquiry {
        question: NonEmptyStr::new(
            "Как крейты DACC попадают в сторонний проект, чья политика зависимостей допускает только crates.io, так, чтобы CI проекта собирал реестр с закреплённой версией?"
        ),
        produces: InquiryOutcome::Adr,
        timebox_days: NonZeroU16::new(2).unwrap(),
    },
    taxon: taxon!(Subsystem, Methodology),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "ADR: способ доставки — публикация, git-зависимость с исключением в политике проекта или копия в дереве проекта — с ценой выпуска новой версии, требованиями к политике и CI проекта и перечнем необратимого."
    ),
);
