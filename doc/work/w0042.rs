use crate::taxonomy::Subsystem;
use slipway_core::{taxon, BlastRadius, NonEmptyStr};
use slipway_work::WorkOrigin;

slipway_work::work!(42,
    title: NonEmptyStr::new("Структурный вердикт в доказательстве и в журнале"),
    slice: crate::slice::s0014,
    origin: WorkOrigin::Decision(crate::adr::a0022),
    taxon: taxon!(Subsystem, Work),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Событие проверки несёт пройденные шаги, их общее число, пропущенные, число прошедших атак и минимальный тулчейн; прежние события с прозаическим вердиктом читаются без правки, и сценарий показывает чтение обоих видов."
    ),
);
