use crate::taxonomy::Subsystem;
use slipway_core::{nonempty_str, taxon};
use slipway_knowledge::{Breaking, DocStatus};

slipway_knowledge::adr!(
    title: "Проект переименован в DACC — Docs As Compiled Code",
    status: DocStatus::Active,
    subsystems: &[taxon!(Subsystem, Methodology)],
    context: r"
        Имя slipway занято на crates.io другим проектом — утилитой деплоя по ssh
        (github.com/grahov/slipway), к методологии отношения не имеющей. Наши
        крейты slipway-* путаются с ним и теряют узнаваемость.
    ",
    decision: r"
        Проект переименовывается в DACC — сокращение «Docs As Compiled Code»,
        которое называет саму идею методологии: документация не просто как код,
        а компилируется. Крейты становятся dacc-core, dacc-knowledge, dacc-work,
        dacc-scan, dacc-derive, dacc-journal, dacc-cli, dacc-access; крейт
        документов — dacc-doc; бинарь — cargo-dacc; настройка — dacc.toml.
        Русское имя «Стапель» остаётся метафорой, а не именем пакета.

        Переименование ломающее: меняются имена крейтов и идентификаторы
        slipway_* в коде. Старые крейты slipway-* на crates.io выводятся через
        yank.
    ",
    trade_offs: &[
        "Плюс: имя уникально и называет инновацию, а не метафору",
        "Плюс: нет путаницы с чужим slipway",
        "Минус: ломающее переименование всех крейтов и идентификаторов",
        "Минус: теряется преемственность имени Slipway в истории",
    ],
    constraints: &[
        "Имена крейтов и идентификаторы — dacc-* / dacc_*.",
        "Старые крейты slipway-* на crates.io — yank после публикации dacc-*.",
    ],
    authors: nonempty_str!["Анищук Сергей"],
    decided_at: slipway_core::date!(2026, 9, 28),
    breaking: Breaking::Yes {
        migration: nonempty_str![
            "Переименовать крейты slipway-* в dacc-*.",
            "Заменить идентификаторы slipway_* на dacc_* в коде и реестре.",
            "Переименовать бинарь в cargo-dacc и настройку в dacc.toml.",
        ],
    },
    code_refs: &[],
    related_rfcs: &[],
);
