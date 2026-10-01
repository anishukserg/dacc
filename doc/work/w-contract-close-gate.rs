use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-contract-close-gate",
    title: NonEmptyStr::new("Закрытие среза требует исполненных контрактов"),
    slice: crate::slice::s_contract_enforcers,
    origin: WorkOrigin::Specification(crate::rfc::rfc_2026_002),
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new("cargo dacc slice close отвергает срез, чья спецификация не имеет ни одной записи invariant! со статусом Enforced и живым якорем enforced_by, или имеет запись, не доведённую до Enforced; отказ назван кодом причины; проверка стоит на закрытии среза, а не на статусе спецификации — итеративный цикл срезов сохранён."),
);
