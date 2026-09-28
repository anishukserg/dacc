use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::{InquiryOutcome, WorkOrigin};
use std::num::NonZeroU16;

dacc_work::work!(63,
    title: NonEmptyStr::new("Как версионировать крейты, чтобы github и crates.io совпадали"),
    slice: crate::slice::s0022,
    origin: WorkOrigin::Inquiry {
        question: NonEmptyStr::new(
            "Крейты опубликованы 0.1.0, но в репозитории нет ни тегов, ни политики поднятия версии: после правок калитки dacc-cli на crates.io остался 0.1.0 со старым кодом. Как версионировать, чтобы версия на crates.io была привязана к коммиту и тегу в github (решение 25)?"
        ),
        produces: InquiryOutcome::Adr,
        timebox_days: NonZeroU16::new(1).unwrap(),
    },
    taxon: taxon!(Subsystem, Methodology),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "ADR: единая версия рабочего пространства, semver для 0.x, тег vX.Y.Z на каждый выпуск и порядок публикации в порядке зависимостей."
    ),
);
