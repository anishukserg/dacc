use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-changelog-0-5-0",
    title: NonEmptyStr::new("CHANGELOG.md с заметками к 0.5.0"),
    slice: crate::slice::s_release_0_5_0,
    origin: WorkOrigin::Toil {
        justification: NonEmptyStr::new(
            "CHANGELOG пишется вручную до порождения из реестра upgrade!: потребитель видит, что нового в 0.5.0 и что менять в машинных формах"
        ),
    },
    taxon: taxon!(Subsystem, Methodology),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "CHANGELOG.md с секцией 0.5.0: work next с допуском и WIP-лимитом, declare_baseline! и ратчет, отказы dacc-error с legitimate, усечение с shown и total, честный разбор записей; миграции библиотеки нет."
    ),
);
