use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-gate-delegation-once",
    title: NonEmptyStr::new("Условие делегирования gate_command вычисляется один раз"),
    slice: crate::slice::s_pilot_gate_findings_2,
    origin: WorkOrigin::Divergence {
        specification: crate::rfc::rfc_2026_002,
        violated: NonEmptyStr::new(
            "Решение 28 объявляет команду проекта одной проверкой с одним исполнителем, но условие «gate_command задана» повторяется в каждом из шести шагов cargo и в счётчике шагов: решение о делегировании размазано по коду и при добавлении шага может разойтись."
        ),
    },
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "Признак делегирования вычисляется один раз перед командой проекта, шесть стандартных шагов cargo сверяются с ним; поведение калитки не меняется, тесты зелёные."
    ),
);
