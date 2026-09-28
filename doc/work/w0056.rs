use crate::taxonomy::Subsystem;
use slipway_core::{taxon, BlastRadius, NonEmptyStr};
use slipway_work::WorkOrigin;

slipway_work::work!(56,
    title: NonEmptyStr::new("Библиотечная точка входа раскладки реестра"),
    slice: crate::slice::s0018,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_027),
    taxon: taxon!(Subsystem, Scan),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "slipway_scan::emit_registry делает скан и запись всех порождённых файлов; build.rs потребителя — один вызов точки входа; сценарий показывает, что демо-build.rs сводится к точке входа и порождённое совпадает."
    ),
);
