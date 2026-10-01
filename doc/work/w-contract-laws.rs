use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-contract-laws",
    title: NonEmptyStr::new("Законы форматов именованы и покрыты тестами"),
    slice: crate::slice::s_contract_enforcers,
    origin: WorkOrigin::Specification(crate::rfc::rfc_2026_002),
    taxon: taxon!(Subsystem, Work),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new("Законы журнала именованы contract-тестами с названным постусловием: round-trip записи, идемпотентность вывода события, монотонность свёртки; новые зависимости не добавляются — законы покрыты параметрическими тестами на существующих крейтах; набор контролируется cargo-mutants."),
);
