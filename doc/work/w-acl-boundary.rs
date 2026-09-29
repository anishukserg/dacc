use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::{InquiryOutcome, WorkOrigin};
use std::num::NonZeroU16;

dacc_work::work!("w-acl-boundary",
    title: NonEmptyStr::new("Контрактная граница направлений зависимостей"),
    slice: crate::slice::s_typed_checks,
    origin: WorkOrigin::Inquiry {
        question: NonEmptyStr::new(
            "Как выразить границу «этот слой не импортирует тот» и «реализация типа только в этих крейтах» типом, чтобы нарушение невыразимо, а не поймано grep'ом (как pg_leak_deny у AngaraBase)?"
        ),
        produces: InquiryOutcome::Adr,
        timebox_days: NonZeroU16::new(2).unwrap(),
    },
    taxon: taxon!(Subsystem, Methodology),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "ADR: обобщение declare_channels! до границы направлений зависимостей и sealed-реализаций, проверяемой на сборке."
    ),
);
