use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-red-before-exists",
    title: NonEmptyStr::new("RedBefore называет существующий и исполняемый тест"),
    slice: crate::slice::s_proof_honesty,
    origin: WorkOrigin::Divergence { specification: crate::rfc::rfc_2026_002, limitation: crate::limitation::l_red_before_claim },
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new("cargo dacc work land проверяет, что предмет --red-before встречается среди имён тестов дерева коммита, и что калитка исполняла тесты: вымышленное имя отвергается кодом причины; падение до починки остаётся заявлением — граница названа ограничением l-red-before-claim."),
);
