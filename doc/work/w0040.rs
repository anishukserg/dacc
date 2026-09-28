use crate::taxonomy::Subsystem;
use slipway_core::{taxon, BlastRadius, NonEmptyStr};
use slipway_work::WorkOrigin;

slipway_work::work!(40,
    title: NonEmptyStr::new("Вердикты и отказы в JSON"),
    slice: crate::slice::s0014,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_022),
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "Калитка, коммит, проверка сообщения и состояние работы принимают --format json и печатают вердикт полями; человекочитаемый вывод остаётся по умолчанию; сценарии закрепляют снимок каждой формы."
    ),
);
