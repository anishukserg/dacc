use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!(18,
    title: NonEmptyStr::new("cargo dacc gate"),
    slice: crate::slice::s0006,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_014),
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "cargo dacc gate выполняет шаги прежней калитки, кроме проверки синтаксиса скриптов, с тем же видом вердикта и кодами возврата; cargo запускается без переменных GIT_* и без унаследованной RUSTUP_TOOLCHAIN; на рабочем дереве DACC вердикт совпадает со скриптом калитки; сценарии внешних имён, markdown, отсутствующего манифеста и явного дерева — интеграционные тесты."
    ),
);
