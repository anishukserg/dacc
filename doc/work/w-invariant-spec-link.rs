use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-invariant-spec-link",
    title: NonEmptyStr::new("Инвариант знает свою спецификацию ссылкой"),
    slice: crate::slice::s_contract_enforcers,
    origin: WorkOrigin::Specification(crate::rfc::rfc_2026_001),
    taxon: taxon!(Subsystem, Knowledge),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new("Поле specification типа RfcRef в записи invariant! — инвариант обязан назвать спецификацию, чей контракт держит; запись без спецификации не компилируется; существующие записи реестра приведены к новой схеме."),
);
