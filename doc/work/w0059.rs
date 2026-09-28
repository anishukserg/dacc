use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!(59,
    title: NonEmptyStr::new("gate_command заменяет шаги cargo, а не добавляется"),
    slice: crate::slice::s0019,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_028),
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "При заданной gate_command калитка пропускает шаги 5–7, 10, 11, 13 и держит 1–3, 8, 9, 12; сценарий показывает, что команда проекта не дублирует fmt/clippy/test, а атаки всё ещё прогоняются."
    ),
);
