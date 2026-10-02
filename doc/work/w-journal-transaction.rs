use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-journal-transaction",
    title: NonEmptyStr::new("Откат журнала чистит индекс, лок коротко ждёт"),
    slice: crate::slice::s_agent_robustness,
    origin: WorkOrigin::Divergence {
        specification: crate::rfc::rfc_2026_002,
        limitation: crate::limitation::l_journal_rollback,
    },
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Отказ коммита события откатывает файл журнала и снимает его из индекса git; межпроцессный лок записи ждёт ограниченное время и только потом отказывает кодом lock-not-acquired; каждый пункт доказан тестом."
    ),
);
