use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-access-limits-xml",
    title: NonEmptyStr::new("Потолки ответов и формат xml"),
    slice: crate::slice::s_access_contract,
    origin: WorkOrigin::Specification(crate::rfc::rfc_2026_003),
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new("Ответ map не превышает 8 КБ независимо от размера проекта, и усечение явно названо в ответе; map и where отвечают xml наравне с json; потолок и усечение покрыты тестом на большом реестре."),
);
