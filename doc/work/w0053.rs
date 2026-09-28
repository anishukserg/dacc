use crate::taxonomy::Subsystem;
use slipway_core::{taxon, BlastRadius, NonEmptyStr};
use slipway_work::WorkOrigin;

slipway_work::work!(53,
    title: NonEmptyStr::new("Slug-идентификаторы документов знания: скан, схемы, переименование реестра"),
    slice: crate::slice::s0017,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_023),
    taxon: taxon!(Subsystem, Knowledge),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Макросы adr!/rfc! без числового аргумента; поле id удалено из записей знания; AdrRef/RfcRef/SupersededRef/BreakingRef несут slug; скан принимает slug из имени файла и отвергает не-slug; реестр и демо переименованы, ссылки обновлены, workspace зелёный."
    ),
);
