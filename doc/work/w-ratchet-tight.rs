use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-ratchet-tight",
    title: NonEmptyStr::new("Baseline держит точный размер долга"),
    slice: crate::slice::s_agent_robustness,
    origin: WorkOrigin::Divergence {
        specification: crate::rfc::rfc_2026_001,
        limitation: crate::limitation::l_ratchet_slack,
    },
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "Проверка бюджета файла сравнивает размер с заморозкой в точности: уменьшение долга требует явного обновления заморозки, возврат к замороженному размеру после уменьшения — отказ теста, и правило проверяется юнит-тестом на люфте."
    ),
);
