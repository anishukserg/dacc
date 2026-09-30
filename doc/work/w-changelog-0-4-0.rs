use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-changelog-0-4-0",
    title: NonEmptyStr::new("CHANGELOG.md с upgrade notes для 0.4.0"),
    slice: crate::slice::s_upgrade_guide,
    origin: WorkOrigin::Toil {
        justification: NonEmptyStr::new(
            "Пока upgrade! не реализован, CHANGELOG пишется вручную: потребитель при bump'е видит, что менять (scan_journal, work start, journal.toml). Позже он порождается из реестра."
        ),
    },
    taxon: taxon!(Subsystem, Methodology),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "CHANGELOG.md с секцией Upgrade notes для 0.4.0: scan_journal принимает два пути, work start не коммитит, журнал — одна append-only запись."
    ),
);
