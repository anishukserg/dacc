use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::{InquiryOutcome, WorkOrigin};
use std::num::NonZeroU16;

dacc_work::work!("w-project-gate-inquiry",
    title: NonEmptyStr::new("Собственный гейт проекта без двойного прогона"),
    slice: crate::slice::s_pilot_gate_findings,
    origin: WorkOrigin::Inquiry {
        question: NonEmptyStr::new(
            "Как проект встраивает собственный гейт через gate_command (например, cargo xtask ci), не запуская дважды форматирование, clippy и тесты, которые калитка DACC делает сама на шагах 5–13 (решение 20)?"
        ),
        produces: InquiryOutcome::Adr,
        timebox_days: NonZeroU16::new(2).unwrap(),
    },
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "ADR: что остаётся DACC, что делегируется команде проекта и как исключается двойной прогон пересекающихся шагов калитки."
    ),
);
