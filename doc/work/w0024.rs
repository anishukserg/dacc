use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!(24,
    title: NonEmptyStr::new("Журнал DACC восстановлен из истории"),
    slice: crate::slice::s0007,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_015),
    taxon: taxon!(Subsystem, Work),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "Работы DACC с коммитами по трейлеру записаны в журнал как приземлённые из истории; срезы без незавершённых работ закрыты; состояние плана выводится свёрткой, а не догадкой по git log."
    ),
);
