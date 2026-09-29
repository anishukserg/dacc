use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-branch-protection",
    title: NonEmptyStr::new("Защита ветки master правилами GitHub"),
    slice: crate::slice::s_branch_protection,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_032),
    taxon: taxon!(Subsystem, Methodology),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "CONTRIBUTING.md называет защиту ветки master: требование прохождения CiGate (CI из решения 18) перед merge, запрет прямой записи и шаги настройки GitHub."
    ),
);
