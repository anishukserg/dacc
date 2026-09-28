use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!(33,
    title: NonEmptyStr::new("dacc.toml: пути реестра, правила темы, шаблоны служебных коммитов"),
    slice: crate::slice::s0012,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_020),
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "Инструмент читает dacc.toml тем же плоским разбором, что и события журнала: пути реестра, набор типов и предел темы, шаблоны служебных тем, ссылку на правила проекта; без файла поведение прежнее; сценарий на чужой раскладке проходит."
    ),
);
