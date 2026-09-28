use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!(19,
    title: NonEmptyStr::new("Хуки через cargo dacc, без скриптов bash"),
    slice: crate::slice::s0006,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_014),
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "Хуки в .githooks — однострочники, вызывающие cargo dacc hook; pre-commit выгружает дерево коммита и запускает калитку инструментом из этого дерева; pre-push проверяет ветки архива, зависимости вершины и внешние имена в истории; hooks install ставит хуки в другом проекте; каталога tools/ нет; сценарии pre-commit и pre-push — интеграционные тесты."
    ),
);
