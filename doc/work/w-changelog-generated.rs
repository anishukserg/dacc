use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-changelog-generated",
    title: NonEmptyStr::new("CHANGELOG: Upgrade notes порождается командой"),
    slice: crate::slice::s_changelog_generated,
    origin: WorkOrigin::Toil {
        justification: NonEmptyStr::new(
            "Рукописная секция Upgrade notes расходится с реестром upgrade!; секция заменяется выводом cargo dacc upgrade 0.4.0."
        ),
    },
    taxon: taxon!(Subsystem, Methodology),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "Секция Upgrade notes CHANGELOG.md — вывод cargo dacc upgrade 0.4.0 с пометкой, что она порождается командой."
    ),
);
