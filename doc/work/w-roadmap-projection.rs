use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-roadmap-projection",
    title: NonEmptyStr::new("Дорожная карта как первая проекция реестра"),
    slice: crate::slice::s_registry_projection,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_021),
    taxon: taxon!(Subsystem, Access),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "Дорожная карта порождается скомпилированным реестром: направления, их срезы с признаком закрытия и работы с состояниями из журнала; карта несёт коммит, из которого собрана; схема документов при этом не меняется."
    ),
);
