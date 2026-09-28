use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-registry-export",
    title: NonEmptyStr::new("Экспорт реестра: JSON для инструментов, XML для среза"),
    slice: crate::slice::s_registry_projection,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_021),
    taxon: taxon!(Subsystem, Access),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Крейт документов отдаёт JSON всего графа и XML целевого среза по вопросу; сериализаторы без внешних зависимостей, вывод несёт версию схемы, а снимки обоих форматов закреплены сценариями."
    ),
);
