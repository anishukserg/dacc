use crate::taxonomy::Subsystem;
use slipway_core::{taxon, BlastRadius, NonEmptyStr};
use slipway_work::WorkOrigin;

slipway_work::work!(37,
    title: NonEmptyStr::new("Экспорт реестра: JSON для инструментов, XML для среза"),
    slice: crate::slice::s0013,
    origin: WorkOrigin::Decision(crate::adr::a0021),
    taxon: taxon!(Subsystem, Access),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Крейт документов отдаёт JSON всего графа и XML целевого среза по вопросу; сериализаторы без внешних зависимостей, вывод несёт версию схемы, а снимки обоих форматов закреплены сценариями."
    ),
);
