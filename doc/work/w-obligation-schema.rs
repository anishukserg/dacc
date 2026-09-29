use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-obligation-schema",
    title: NonEmptyStr::new("Запись Obligation и макрос obligation!"),
    slice: crate::slice::s_obligation_registry,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_035),
    taxon: taxon!(Subsystem, Work),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Obligation с полями id, title и обязательным discharged_when; obligation! макрос; скан порождает ObligationRef и ALL_OBLIGATIONS; WorkOrigin::Obligation(ObligationRef) — новое происхождение работы."
    ),
);
