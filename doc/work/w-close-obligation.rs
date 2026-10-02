use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-close-obligation",
    title: NonEmptyStr::new("slice close не закрывает срез с непогашенным обязательством"),
    slice: crate::slice::s_proof_close_honesty,
    origin: WorkOrigin::Divergence { specification: crate::rfc::rfc_2026_002, limitation: crate::limitation::l_slice_obligation_ignored },
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new("slice close читает обязательства реестра и происхождения работ, отказывает кодом obligation-not-redeemed, пока обязательство работ среза не погашено приземлённой работой WorkOrigin::Obligation, и закрывает срез после погашения; сценарий падал до починки"),
);
