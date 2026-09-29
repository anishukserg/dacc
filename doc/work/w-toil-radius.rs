use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-toil-radius",
    title: NonEmptyStr::new("Toil не выходит за пределы Local"),
    slice: crate::slice::s_toil_radius,
    origin: WorkOrigin::Divergence {
        specification: crate::rfc::rfc_2026_002,
        limitation: crate::limitation::l_toil_radius,
    },
    taxon: taxon!(Subsystem, Work),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Работа с WorkOrigin::Toil не может объявить радиус выше Local; нарушение — ошибка вычисления константы, а не находка валидатора."
    ),
);
