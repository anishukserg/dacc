use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-limitations-registry",
    title: NonEmptyStr::new("Реестр известных ограничений со сверкой кода"),
    slice: crate::slice::s_typed_checks,
    origin: WorkOrigin::Divergence {
        specification: crate::rfc::rfc_2026_002,
        violated: NonEmptyStr::new(
            "Работы с происхождением Divergence называют расхождение текстом, но реестр не сверяется с кодом: необъявленное расхождение не ловится сборкой."
        ),
    },
    taxon: taxon!(Subsystem, Work),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Типизированный реестр limitation! и скан, который находит call-site'ы и требует объявления каждого: необъявленный отказ — ошибка сборки."
    ),
);
