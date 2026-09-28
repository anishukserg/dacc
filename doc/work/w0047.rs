use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::{InquiryOutcome, WorkOrigin};
use std::num::NonZeroU16;

dacc_work::work!(47,
    title: NonEmptyStr::new("Как новый потребитель получает раскладку реестра"),
    slice: crate::slice::s0008,
    origin: WorkOrigin::Inquiry {
        question: NonEmptyStr::new(
            "Как новый потребитель получает раскладку реестра так, чтобы она не копировалась руками и не расходилась у двух проектов: порождением из инструмента, библиотечной точкой входа вместо шаблона или тем и другим?"
        ),
        produces: InquiryOutcome::Adr,
        timebox_days: NonZeroU16::new(3).unwrap(),
    },
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "ADR: что порождает инструмент, что остаётся библиотечной точкой входа, чем проверяется расхождение раскладки у потребителя и какова цена смены API скана для уже порождённого."
    ),
);
