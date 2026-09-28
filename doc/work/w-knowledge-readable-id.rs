use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::{InquiryOutcome, WorkOrigin};
use std::num::NonZeroU16;

dacc_work::work!("w-knowledge-readable-id",
    title: NonEmptyStr::new("Как адресовать документы знания читаемым идентификатором"),
    slice: crate::slice::s_knowledge_slugs,
    origin: WorkOrigin::Inquiry {
        question: NonEmptyStr::new(
            "Как адресовать документы слоя знания, чтобы идентификатор был читаемым, определялся командой-потребителем и не ломал инъективность констант-ссылок и их проверку компилятором, при этом не трогая append-only слой работы?"
        ),
        produces: InquiryOutcome::Adr,
        timebox_days: NonZeroU16::new(2).unwrap(),
    },
    taxon: taxon!(Subsystem, Knowledge),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "ADR: идентификатор — slug из имени файла; алфавит slug; судьба числового поля id; граница слоя работы; миграция ссылок."
    ),
);
