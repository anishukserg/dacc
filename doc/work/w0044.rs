use crate::taxonomy::Subsystem;
use slipway_core::{taxon, BlastRadius, NonEmptyStr};
use slipway_work::WorkOrigin;

slipway_work::work!(44,
    title: NonEmptyStr::new("Первый коммит в пустом репозитории делается инструментом"),
    slice: crate::slice::s0011,
    origin: WorkOrigin::Divergence {
        specification: crate::rfc::r0002,
        violated: NonEmptyStr::new(
            "Правила коммита исполнимы самим инструментом с первого коммита: обходить их отдельным git add не требуется (решение 8)."
        ),
    },
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "Сценарий, падавший до починки, проходит: в репозитории без коммитов инструмент сам стейджит перечисленные пути и проверяет основание по дереву будущего коммита; форма темы по-прежнему проверяется до блокировки."
    ),
);
