use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-observable-violations",
    title: NonEmptyStr::new("Нарушение называет сущность, файл, строку, связь и способ исправления"),
    slice: crate::slice::s_observable_check_limits,
    origin: WorkOrigin::Divergence {
        specification: crate::rfc::rfc_2026_002,
        limitation: crate::limitation::l_observable_violations,
    },
    taxon: taxon!(Subsystem, Scan),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Каждое нарушение скана называет сущность, файл, строку, сломанную связь и способ исправления; отсутствующая строка или подсказка «как чинить» невыразимы."
    ),
);
