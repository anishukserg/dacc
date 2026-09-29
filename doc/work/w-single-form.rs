use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-single-form",
    title: NonEmptyStr::new("Один дом для формы правила, дубли запрещены"),
    slice: crate::slice::s_registry_hygiene,
    origin: WorkOrigin::Divergence {
        specification: crate::rfc::rfc_2026_001,
        limitation: crate::limitation::l_single_form,
    },
    taxon: taxon!(Subsystem, Methodology),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "Механизм детекта дублей формы правила: копия — ошибка сборки, а не находка ревью."
    ),
);
