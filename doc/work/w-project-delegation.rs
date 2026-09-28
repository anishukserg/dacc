use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-project-delegation",
    title: NonEmptyStr::new("Делегирование проверок проекту и настраиваемый пол атак"),
    slice: crate::slice::s_tool_for_other_projects,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_020),
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "Команда проекта выполняется шагом калитки, и её отказ — отказ калитки; при заданной команде разбор сообщения делегируется ей, а трейлер основания по-прежнему проверяет DACC; пол атакующих doctest берётся из настройки."
    ),
);
