use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-gate-delegation-once",
    title: NonEmptyStr::new("Условие делегирования gate_command вычисляется один раз"),
    slice: crate::slice::s_pilot_gate_findings_2,
    origin: WorkOrigin::Divergence {
        specification: crate::rfc::rfc_2026_002,
        limitation: crate::limitation::l_gate_delegation_once,
    },
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "Признак делегирования вычисляется один раз перед командой проекта, шесть стандартных шагов cargo сверяются с ним; поведение калитки не меняется, тесты зелёные."
    ),
);
