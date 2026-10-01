use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-journal-lock",
    title: NonEmptyStr::new("Запись журнала под межпроцессным локом"),
    slice: crate::slice::s_gate_integrity,
    origin: WorkOrigin::Specification(crate::rfc::rfc_2026_002),
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new("record_and_commit берёт межпроцессный лок записи журнала до события и коммита; параллельный процесс получает отказ с кодом причины, а не теряет событие; тест со вторым процессом показывает отсутствие коллизии."),
);
