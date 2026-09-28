use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-common-git-dir",
    title: NonEmptyStr::new("Доказательство и список имён — от общего каталога git"),
    slice: crate::slice::s_tool_defects,
    origin: WorkOrigin::Divergence {
        specification: crate::rfc::rfc_2026_002,
        violated: NonEmptyStr::new(
            "Доказательство готовности принадлежит репозиторию: калитка, пройденная в изолированной рабочей копии, остаётся доказательством того же дерева (решения 4 и 15)."
        ),
    },
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "Сценарий, падавший до починки, проходит: коммит в связанной рабочей копии даёт доказательство, по которому приземляет основная; список внешних имён из общего каталога виден в любой копии; блокировка и выгрузка дерева остаются приватными для копии."
    ),
);
