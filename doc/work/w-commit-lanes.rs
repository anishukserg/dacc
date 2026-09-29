use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::{InquiryOutcome, WorkOrigin};
use std::num::NonZeroU16;

dacc_work::work!("w-commit-lanes",
    title: NonEmptyStr::new("Лейны коммитов: правка инструмента против правки дерева"),
    slice: crate::slice::s_registry_hygiene,
    origin: WorkOrigin::Inquiry {
        question: NonEmptyStr::new(
            "Как различать коммит «правка инструмента» и «правка реестра/продукта» лейном, влияющим на выбор яруса калитки (как commit_lane у AngaraBase)?"
        ),
        produces: InquiryOutcome::Adr,
        timebox_days: NonZeroU16::new(2).unwrap(),
    },
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "ADR: лейн коммита в журнале, различающий радиус и выбор яруса калитки."
    ),
);
