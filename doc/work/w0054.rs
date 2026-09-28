use crate::taxonomy::Subsystem;
use slipway_core::{taxon, BlastRadius, NonEmptyStr};
use slipway_work::WorkOrigin;

slipway_work::work!(54,
    title: NonEmptyStr::new("Инлайн-ссылки на якоря в прозе решений"),
    slice: crate::slice::s0001,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_024),
    taxon: taxon!(Subsystem, Scan),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Скан извлекает из прозы ссылки формы `[id]`, сверяет с разметкой и отказывает на несуществующий якорь; сценарий показывает, что удаление разметки ломает ссылку, а свободное упоминание имени символа не проверяется."
    ),
);
