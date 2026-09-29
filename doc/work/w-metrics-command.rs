use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-metrics-command",
    title: NonEmptyStr::new("cargo dacc metrics выводит счётчики пилота"),
    slice: crate::slice::s_measurable,
    origin: WorkOrigin::Specification(crate::rfc::rfc_2026_003),
    taxon: taxon!(Subsystem, Access),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Счётчики пилота — коммиты с основанием, журнальные записи, радиусы, статусы инвариантов — выводятся одной командой из реестра, а не считаются руками."
    ),
);
