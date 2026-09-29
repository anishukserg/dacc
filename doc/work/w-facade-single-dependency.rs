use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::{InquiryOutcome, WorkOrigin};
use std::num::NonZeroU16;

dacc_work::work!("w-facade-single-dependency",
    title: NonEmptyStr::new("Как фасад становится единственной зависимостью"),
    slice: crate::slice::s_facade_single_dependency,
    origin: WorkOrigin::Inquiry {
        question: NonEmptyStr::new(
            "Фасад dacc реэкспортирует имена, но $crate макросов и порождённый сканом код ссылаются на крейты L1 (dacc-core, dacc-knowledge, dacc-work), поэтому потребитель подключает всю пачку. Как сделать разметку реестра одной зависимостью: параметризацией пути в порождённом коде, процедурным макросом или иначе?"
        ),
        produces: InquiryOutcome::Adr,
        timebox_days: NonZeroU16::new(2).unwrap(),
    },
    taxon: taxon!(Subsystem, Methodology),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "ADR: способ, которым порождённый код и макросы ссылаются на фасад dacc, чтобы разметка реестра требовала одну зависимость."
    ),
);
