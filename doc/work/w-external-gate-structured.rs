use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-external-gate-structured",
    title: NonEmptyStr::new("Структурный вердикт внешней калитки в доказательстве"),
    slice: crate::slice::s_machine_contract,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_022),
    taxon: taxon!(Subsystem, Work),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Хук записывает структурный вердикт и для внешней калитки — из машинного вывода `--format json`; событие gate несёт структуру, прежние доказательства с прозой читаются без правки."
    ),
);
