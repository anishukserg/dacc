use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-gate-tier-split",
    title: NonEmptyStr::new("Разбить калитку на ярус коммита и полный ярус"),
    slice: crate::slice::s_gate_tiers,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_033),
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "`gate` без флага исполняет ярус коммита, `--full` добавляет MSRV и зависимости; вердикт яруса коммита — без msrv, полного — с msrv; хук pre-commit и CI исполняют полный ярус."
    ),
);
