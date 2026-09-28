use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!(38,
    title: NonEmptyStr::new("Каналы публикации в схеме документов"),
    slice: crate::slice::s0013,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_021),
    taxon: taxon!(Subsystem, Knowledge),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Продукт объявляет значения каналов, документ несёт канал, и канал вне объявленных не компилируется; экспорт канала не выпускает документ другого канала и отвергает ссылку через границу канала — сценарий показывает отказ."
    ),
);
