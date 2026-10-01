use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-evidence-kinds",
    title: NonEmptyStr::new("Виды доказательства RedBefore, MutationProof и AntiVacuum — тип события"),
    slice: crate::slice::s_anti_vacuous_evidence,
    origin: WorkOrigin::Divergence { specification: crate::rfc::rfc_2026_002, limitation: crate::limitation::l_evidence_kinds },
    taxon: taxon!(Subsystem, Work),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new("Тип Proof в dacc-journal с тремя видами и непустым предметом; событие landed несёт proofs плоскими полями red_before, mutation_proof и anti_vacuum; разбор отвергает пустой предмет и повтор ключа, прежние события читаются без правки; запись, разбор и обратная совместимость покрыты тестами."),
);
