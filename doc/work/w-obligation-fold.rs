use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-obligation-fold",
    title: NonEmptyStr::new("Обязательство в свёртке журнала"),
    slice: crate::slice::s_obligation_registry,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_035),
    taxon: taxon!(Subsystem, Work),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Subject::Obligation и Journal.obligations; свёртка показывает непогашенное обязательство, погашенное — приземлённой работой с происхождением WorkOrigin::Obligation."
    ),
);
