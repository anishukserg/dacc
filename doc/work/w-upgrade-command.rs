use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-upgrade-command",
    title: NonEmptyStr::new("cargo dacc upgrade выводит шаги повышения версии"),
    slice: crate::slice::s_upgrade_guide,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_044),
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "cargo dacc upgrade <from> <to> выводит шаги subject + how из реестра upgrade!; без аргументов — все шаги по версиям."
    ),
);
