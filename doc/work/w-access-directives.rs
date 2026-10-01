use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-access-directives",
    title: NonEmptyStr::new("Файлы-директивы агентов: cargo dacc emit all"),
    slice: crate::slice::s_access_first_touch,
    origin: WorkOrigin::Specification(crate::rfc::rfc_2026_003),
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new("cargo dacc emit all пишет файлы-директивы агентов с правилами навигации, ритуалами коммита и работ и перечнем того, что команды отвечают; повторный запуск идентичен закоммиченному файлу — это сверяет тест."),
);
