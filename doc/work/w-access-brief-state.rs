use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-access-brief-state",
    title: NonEmptyStr::new("Сводка и изменения дня: brief и state"),
    slice: crate::slice::s_access_contract,
    origin: WorkOrigin::Specification(crate::rfc::rfc_2026_003),
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new("cargo dacc brief отдаёт постоянную сводку — направления, открытые срезы и открытые работы; cargo dacc state несёт только изменяющееся за день и не превышает 2 КБ; усечение явное; оба ответа — текстом и json."),
);
