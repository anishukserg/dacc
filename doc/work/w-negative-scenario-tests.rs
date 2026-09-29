use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-negative-scenario-tests",
    title: NonEmptyStr::new("Отрицательные сценарии скана проверены тестами"),
    slice: crate::slice::s_observable_check_limits,
    origin: WorkOrigin::Specification(crate::rfc::rfc_2026_002),
    taxon: taxon!(Subsystem, Scan),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Отрицательные сценарии — удалённый якорь, переименование, дубликат идентификатора, устаревшая ссылка, некорректный переход состояния — проверены тестами, и каждый называет способ исправления."
    ),
);
