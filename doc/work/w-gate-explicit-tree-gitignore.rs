use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-gate-explicit-tree-gitignore",
    title: NonEmptyStr::new("Явное дерево калитки уважает .gitignore"),
    slice: crate::slice::s_pilot_gate_findings_2,
    origin: WorkOrigin::Divergence {
        specification: crate::rfc::rfc_2026_002,
        limitation: crate::limitation::l_gate_explicit_tree_gitignore,
    },
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "Обход подкаталога репозитория берёт тот же список git ls-files --cached --others --exclude-standard, ограниченный каталогом; сценарий показывает, что файл в игнорируемом каталоге явного дерева калитка не видит, а тот же текст в неигнорируемом файле — видит."
    ),
);
