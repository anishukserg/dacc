use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!(67,
    title: NonEmptyStr::new("Slug-идентификаторы слоя работы: схемы, скан, журнал, переименование"),
    slice: crate::slice::s0024,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_031),
    taxon: taxon!(Subsystem, Work),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Макросы thrust!/slice!/work! без числового аргумента; поле id удалено из записей работы; ThrustRef/SliceRef/WorkRef и Subject несут slug; скан принимает slug из имени файла; файлы thrust/, slice/, work/ и журнал переименованы; трейлеры и команды cargo dacc work/slice работают по slug; workspace зелёный."
    ),
);
