use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-enforced-needs-tests",
    title: NonEmptyStr::new("Enforced требует якоря на исполняемый тест"),
    slice: crate::slice::s_proof_honesty,
    origin: WorkOrigin::Specification(crate::rfc::rfc_2026_002),
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new("Закрытие среза отвергает Enforced-инвариант с пустым tests: статус требует непустого списка якорей на исполняемые тесты; i-work-origin переведён на тестовый якорь, его тест исполняется калиткой; отказ назван кодом причины."),
);
