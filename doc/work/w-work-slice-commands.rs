use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-work-slice-commands",
    title: NonEmptyStr::new("Команды cargo dacc work и slice"),
    slice: crate::slice::s_work_journal,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_015),
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "work start, land, drop и state и slice close пишут события и коммитят их по правилам коммитов; land отказывает без доказательства для дерева коммита работы; коммит закрытия среза несёт трейлер Dacc-Slice."
    ),
);
