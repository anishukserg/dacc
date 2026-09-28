use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::{InquiryOutcome, WorkOrigin};
use std::num::NonZeroU16;

dacc_work::work!("w-registry-projection-choice",
    title: NonEmptyStr::new("Какую проекцию реестра мы публикуем"),
    slice: crate::slice::s_registry_projection,
    origin: WorkOrigin::Inquiry {
        question: NonEmptyStr::new(
            "Что публикуется из реестра для людей и для агентов, чем публикуемое держится согласованным с деревом, прошедшим калитку, и что делает публикацию внутреннего документа невыразимой?"
        ),
        produces: InquiryOutcome::Adr,
        timebox_days: NonZeroU16::new(3).unwrap(),
    },
    taxon: taxon!(Subsystem, Access),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "ADR: источник проекции, формат для инструментов и формат для контекста агента, поле видимости документа, шаг свежести в калитке, место и правило публикации, выбранный рендерер и цена отказа от него."
    ),
);
