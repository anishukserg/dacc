use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-readme-published-state",
    title: NonEmptyStr::new("README отражает опубликованное состояние"),
    slice: crate::slice::s_doc_actualization,
    origin: WorkOrigin::Toil {
        justification: NonEmptyStr::new(
            "Статус README устарел после публикации 0.1.1 и появления dacc-access: «крейты не опубликованы» и «слой доступа не реализован» уже неверны."
        ),
    },
    taxon: taxon!(Subsystem, Methodology),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "README перечисляет dacc-access в таблице крейтов и в «Статусе» говорит: крейты опубликованы в crates.io, экспорт реестра в JSON и XML реализован, контракт команд map/where/ls/show (RFC-0003) не реализован."
    ),
);
