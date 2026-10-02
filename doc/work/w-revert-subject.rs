use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-revert-subject",
    title: NonEmptyStr::new("Revert проходит форму темы как отменяемая тема"),
    slice: crate::slice::s_pilot_tool_findings,
    origin: WorkOrigin::Divergence { specification: crate::rfc::rfc_2026_002, limitation: crate::limitation::l_revert_subject },
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new("msg-check принимает Revert \"[ТИП](область): суть\" и проверяет форму внутренней темы: тип, область, точка и предел длины берутся от неё; вложенная обёртка разворачивается; тема без формы внутри отказывается; сценарий падал до починки"),
);
