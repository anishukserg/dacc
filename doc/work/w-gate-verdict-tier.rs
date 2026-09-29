use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-gate-verdict-tier",
    title: NonEmptyStr::new("Структурный вердикт калитки различает ярусы"),
    slice: crate::slice::s_gate_tier_wiring,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_033),
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Поле msrv структурного вердикта события gate стало необязательным: полный ярус несёт msrv, ярус коммита — нет; хук pre-commit исполняет ярус коммита, work land исполняет полный ярус на дереве коммита и отказывает без msrv."
    ),
);
