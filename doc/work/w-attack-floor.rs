use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-attack-floor",
    title: NonEmptyStr::new("Пол измеряет исполненные атаки, а не любые doctest"),
    slice: crate::slice::s_close_knowledge_attacks,
    origin: WorkOrigin::Divergence {
        specification: crate::rfc::rfc_2026_002,
        limitation: crate::limitation::l_attack_floor,
    },
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "Пол считает только doctest, объявленные падающими при компиляции; сценарий показывает, что обычные примеры чтения реестра его не набирают, а удаление атаки роняет калитку."
    ),
);
