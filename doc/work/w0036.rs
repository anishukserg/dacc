use crate::taxonomy::Subsystem;
use slipway_core::{taxon, BlastRadius, NonEmptyStr};
use slipway_work::WorkOrigin;

slipway_work::work!(36,
    title: NonEmptyStr::new("Дорожная карта как первая проекция реестра"),
    slice: crate::slice::s0013,
    origin: WorkOrigin::Decision(crate::adr::a0021),
    taxon: taxon!(Subsystem, Access),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "Дорожная карта порождается скомпилированным реестром: направления, их срезы с признаком закрытия и работы с состояниями из журнала; карта несёт коммит, из которого собрана; схема документов при этом не меняется."
    ),
);
