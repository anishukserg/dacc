use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-upgrade-schema",
    title: NonEmptyStr::new("Тип upgrade! в слое работы и реестре"),
    slice: crate::slice::s_upgrade_guide,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_044),
    taxon: taxon!(Subsystem, Knowledge),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Запись upgrade! (from/to/subject/how) в слое работы: схема Upgrade, макрос upgrade!, ссылка UpgradeRef, скан реестра doc/upgrade/ и записи шагов 0.4.0."
    ),
);
