use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::{InquiryOutcome, WorkOrigin};
use std::num::NonZeroU16;

dacc_work::work!("w-work-layer-slug-inquiry",
    title: NonEmptyStr::new("Как адресовать слой работы читаемым slug-идентификатором"),
    slice: crate::slice::s_work_layer_slugs,
    origin: WorkOrigin::Inquiry {
        question: NonEmptyStr::new(
            "Числовые идентификаторы слоя работы (w0001, s0001, t0001) нечитаемы. Как перевести их на slug из имени файла, не трогая контентно-адресуемое доказательство готовности и не ломая append-only журнал и трейлеры (ADR-023)?"
        ),
        produces: InquiryOutcome::Adr,
        timebox_days: NonZeroU16::new(1).unwrap(),
    },
    taxon: taxon!(Subsystem, Work),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "ADR: slug из имени файла с префиксом t-/s-/w-, макросы без числа, ссылки и Subject на slug, порядок миграции журнала и трейлеров."
    ),
);
