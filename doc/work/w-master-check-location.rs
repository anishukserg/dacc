use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use crate::taxonomy::Subsystem;
use dacc_work::{InquiryOutcome, WorkOrigin};
use std::num::NonZeroU16;

dacc_work::work!("w-master-check-location",
    title: NonEmptyStr::new("Где исполняется проверка на пути в основную ветку при агентной разработке"),
    slice: crate::slice::s_minimal_work_layer,
    origin: WorkOrigin::Inquiry {
        question: NonEmptyStr::new(
            "Что и где проверяется на пути в основную ветку, если код пишут агенты, кто потребляет вердикт и чем он независим от автора?"
        ),
        produces: InquiryOutcome::Adr,
        timebox_days: NonZeroU16::new(5).unwrap(),
    },
    taxon: taxon!(Subsystem, Methodology),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "ADR: определение CiGate через независимость вердикта и его потребителя, а не через место исполнения; сверено с двумя известными схемами — проверкой на закрытии работы без удалённого CI и удалённым CI на каждый push; первый набор проверок для DACC."
    ),
);
