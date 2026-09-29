use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-obligation-criteria",
    title: NonEmptyStr::new("Критерии погашения обязательства — обязательны и типизированы"),
    slice: crate::slice::s_typed_checks,
    origin: WorkOrigin::Divergence {
        specification: crate::rfc::rfc_2026_002,
        violated: NonEmptyStr::new(
            "Obligation несёт только discharged_when (свободный текст): вопрос погашается «когда-нибудь», критерии решения не названы."
        ),
    },
    taxon: taxon!(Subsystem, Work),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Obligation несёт обязательные критерии решения (NonEmpty<NonEmptyStr>), а журнал решений — проекция реестра."
    ),
);
