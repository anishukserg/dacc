use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-first-commit",
    title: NonEmptyStr::new("Первый коммит в пустом репозитории делается инструментом"),
    slice: crate::slice::s_tool_defects,
    origin: WorkOrigin::Divergence {
        specification: crate::rfc::rfc_2026_002,
        limitation: crate::limitation::l_first_commit,
    },
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "Сценарий, падавший до починки, проходит: в репозитории без коммитов инструмент сам стейджит перечисленные пути и проверяет основание по дереву будущего коммита; форма темы по-прежнему проверяется до блокировки."
    ),
);
