use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-gate-gitignore",
    title: NonEmptyStr::new("Обход дерева в калитке уважает .gitignore"),
    slice: crate::slice::s_pilot_gate_findings,
    origin: WorkOrigin::Divergence {
        specification: crate::rfc::rfc_2026_002,
        limitation: crate::limitation::l_gate_gitignore,
    },
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "Обход рабочего дерева перечисляет файлы, которые git считает частью проекта (отслеживаемые и неигнорируемые неотслеживаемые); сценарий показывает, что битый файл в игнорируемом каталоге не останавливает калитку, а в отслеживаемом — останавливает."
    ),
);
