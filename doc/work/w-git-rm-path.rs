use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-git-rm-path",
    title: NonEmptyStr::new("Путь, удалённый через git rm, не коммитился"),
    slice: crate::slice::s_cargo_dacc_tool,
    origin: WorkOrigin::Divergence {
        specification: crate::rfc::rfc_2026_002,
        limitation: crate::limitation::l_git_rm_path,
    },
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "Сценарий, падавший до починки, проходит: путь, удалённый через git rm, коммитится командой commit без предварительного git reset; путь, которого нет ни в рабочем дереве, ни в индексе, ни в HEAD, по-прежнему отвергается как опечатка."
    ),
);
