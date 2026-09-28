use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-refusal-codes",
    title: NonEmptyStr::new("Коды причин отказа"),
    slice: crate::slice::s_machine_contract,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_022),
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "У каждой причины отказа есть стабильный код, он печатается рядом с текстом и входит в машинный вывод; тест отвергает причину без кода и повторное использование кода с другим смыслом."
    ),
);
