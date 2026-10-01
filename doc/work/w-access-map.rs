use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-access-map",
    title: NonEmptyStr::new("Карта реестра одной командой: cargo dacc map"),
    slice: crate::slice::s_access_first_touch,
    origin: WorkOrigin::Specification(crate::rfc::rfc_2026_003),
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new("cargo dacc map печатает карту реестра — направления, срезы с исходами, работы со свёрткой журнала и счётчики решений и спецификаций; --format json отдаёт те же данные полями; ответ идёт из реестра, find, grep и ls для навигации не нужны."),
);
