use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-record-parse",
    title: NonEmptyStr::new("Записи читаются значениями, а не подстроками"),
    slice: crate::slice::s_gate_integrity,
    origin: WorkOrigin::Divergence { specification: crate::rfc::rfc_2026_002, limitation: crate::limitation::l_gate_bypass },
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new("Разбор полей записи снимает комментарии и нормализует пробелы, значение читается сбалансированной группой: check_contract отличает InvariantStatus::Enforced в значении status от текста в rationale и комментарии, пустой enforced_by ловится в любой форме записи, specification читается при любом переносе; атаки на три формы обхода покрыты тестами."),
);
