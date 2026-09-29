use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-architectural-balance-doc",
    title: NonEmptyStr::new("Раздел «Архитектурный баланс» в DACC.md"),
    slice: crate::slice::s_doc_balance,
    origin: WorkOrigin::Toil {
        justification: NonEmptyStr::new(
            "Независимое ревью архитектуры свело механики к таблице «сила → зазор → статус»; перенос в DACC.md с уточнением родов зазора."
        ),
    },
    taxon: taxon!(Subsystem, Methodology),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "DACC.md: раздел «Архитектурный баланс» с разделением реальных зазоров, осознанных компромиссов и инвариантов; версия 1.5."
    ),
);
