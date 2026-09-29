use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-truth-of-now",
    title: NonEmptyStr::new("Контракт и проекция состояния не смешиваются"),
    slice: crate::slice::s_registry_hygiene,
    origin: WorkOrigin::Divergence {
        specification: crate::rfc::rfc_2026_003,
        violated: NonEmptyStr::new(
            "Проекции (roadmap, сайт) порождаются из реестра, но правило «проекция не хранит контракт» не выражено: проекция может начать дублировать нормативное."
        ),
    },
    taxon: taxon!(Subsystem, Access),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "Правило «проекция не хранит контракт» формализовано; проекция всегда порождается из констант реестра."
    ),
);
