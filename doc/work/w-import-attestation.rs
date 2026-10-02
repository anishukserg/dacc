use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-import-attestation",
    title: NonEmptyStr::new("Импорт журнала приземляет только подтверждённое"),
    slice: crate::slice::s_pilot_tool_findings,
    origin: WorkOrigin::Divergence { specification: crate::rfc::rfc_2026_002, limitation: crate::limitation::l_import_invents_landings },
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new("journal import без --land отказывает и перечисляет кандидатов; --land <w> приземляет работу по последнему коммиту с её трейлером, --land <w>=<commit> — по названному коммиту, включая историю до трейлеров; работа без подтверждения остаётся planned; каждый пункт доказан сценарием, падавшим до починки"),
);
