use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-toil-radius",
    title: NonEmptyStr::new("Toil не выходит за пределы Local"),
    slice: crate::slice::s_toil_radius,
    origin: WorkOrigin::Divergence {
        specification: crate::rfc::rfc_2026_002,
        violated: NonEmptyStr::new(
            "происхождение Toil объявляет «рутину», но радиус работы никак не ограничен: оправдание непустой строкой допускает тронуть публичный API без среза и решения"
        ),
    },
    taxon: taxon!(Subsystem, Work),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Работа с WorkOrigin::Toil не может объявить радиус выше Local; нарушение — ошибка вычисления константы, а не находка валидатора."
    ),
);
