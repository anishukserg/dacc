use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-evidence-land",
    title: NonEmptyStr::new("work land требует RedBefore для дефекта и необратимого"),
    slice: crate::slice::s_anti_vacuous_evidence,
    origin: WorkOrigin::Divergence { specification: crate::rfc::rfc_2026_002, limitation: crate::limitation::l_evidence_kinds },
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new("cargo dacc work land принимает --red-before, --mutation-proof и --anti-vacuum; приземление работы с происхождением Divergence или радиусом Persistent/Irreversible отвергается без --red-before с кодом причины; событие landed несёт заявленные виды; отказ и успех покрыты тестами."),
);
