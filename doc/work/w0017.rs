use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!(17,
    title: NonEmptyStr::new("cargo dacc commit и msg-check"),
    slice: crate::slice::s0006,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_014),
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "Крейт dacc-cli без зависимостей выполняет commit и msg-check с теми же кодами возврата и вердиктами, что прежний скрипт коммита; сценарии формы сообщения, основания, путей, журнала и блокировки — интеграционные тесты на временных репозиториях."
    ),
);
