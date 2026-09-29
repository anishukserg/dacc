use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-obligation-gate",
    title: NonEmptyStr::new("Калитка отказывает закрыть срез с непогашенным обязательством"),
    slice: crate::slice::s_obligation_registry,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_035),
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "slice close отказывает, пока у среза есть непогашенное обязательство; сценарий показывает отказ и погашение работой с WorkOrigin::Obligation."
    ),
);
