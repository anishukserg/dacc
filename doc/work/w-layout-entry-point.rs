use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-layout-entry-point",
    title: NonEmptyStr::new("Библиотечная точка входа раскладки реестра"),
    slice: crate::slice::s_publication,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_027),
    taxon: taxon!(Subsystem, Scan),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "dacc_scan::emit_registry делает скан и запись всех порождённых файлов; build.rs потребителя — один вызов точки входа; сценарий показывает, что демо-build.rs сводится к точке входа и порождённое совпадает."
    ),
);
