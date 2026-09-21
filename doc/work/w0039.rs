use crate::taxonomy::Subsystem;
use slipway_core::{taxon, BlastRadius, NonEmptyStr};
use slipway_work::WorkOrigin;

slipway_work::work!(39,
    title: NonEmptyStr::new("Сайт на mdBook и публикация из CI"),
    slice: crate::slice::s0013,
    origin: WorkOrigin::Decision(crate::adr::a0021),
    taxon: taxon!(Subsystem, Access),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "Страницы реестра в Markdown собираются mdBook и публикуются на Pages из CI только с master; каждая страница несёт коммит; шаг калитки отвергает дерево, в котором опубликованное разошлось с порождённым из реестра."
    ),
);
