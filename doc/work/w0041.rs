use crate::taxonomy::Subsystem;
use slipway_core::{taxon, BlastRadius, NonEmptyStr};
use slipway_work::WorkOrigin;

slipway_work::work!(41,
    title: NonEmptyStr::new("Коды причин отказа"),
    slice: crate::slice::s0014,
    origin: WorkOrigin::Decision(crate::adr::a0022),
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "У каждой причины отказа есть стабильный код, он печатается рядом с текстом и входит в машинный вывод; тест отвергает причину без кода и повторное использование кода с другим смыслом."
    ),
);
