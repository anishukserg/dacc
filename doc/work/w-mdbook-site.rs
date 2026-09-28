use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-mdbook-site",
    title: NonEmptyStr::new("Сайт на mdBook и публикация из CI"),
    slice: crate::slice::s_registry_projection,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_021),
    taxon: taxon!(Subsystem, Access),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "Страницы реестра в Markdown собираются mdBook и публикуются на Pages из CI только с master; каждая страница несёт коммит; шаг калитки отвергает дерево, в котором опубликованное разошлось с порождённым из реестра."
    ),
);
