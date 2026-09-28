use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!(22,
    title: NonEmptyStr::new("Доказательство готовности по хэшу дерева без журнала"),
    slice: crate::slice::s0007,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_015),
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "pre-commit после прохождения калитки сохраняет доказательство для хэша дерева коммита без каталога журнала; калитка отвергает изменение или удаление файла журнала и приземление, чей коммит отсутствует или чьё дерево расходится с событием; коммит, меняющий только журнал, переиспользует доказательство."
    ),
);
