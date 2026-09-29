use crate::taxonomy::{channel, Subsystem};
use dacc_core::{nonempty_str, taxon};
use dacc_knowledge::DocStatus;

dacc_knowledge::rfc!(
    title: "Слой доступа",
    status: DocStatus::Draft,
    channel: channel::Public,
    target: &[taxon!(Subsystem, Access), taxon!(Subsystem, Cli)],
    goal: r"
        Человек и агент узнают всё о месте в коде одной командой, не прибегая
        к обходу дерева и текстовому поиску.

        Проверяемо: для файла с размеченным фрагментом команда where
        возвращает подсистему, владельца, регулирующие решения и их запреты,
        обязательные гейты и допуск исполнителя.
    ",
    input_contract: r"
        Реестры знания и работы; журнал; исходники с разметкой.
    ",
    output_contract: r"
        Команды map, state, ls, show, where, refs, find, brief, help.
        Форматы json и xml — версионированный контракт; term и md — нет.
        Каждая команда объявляет стоимость и мутирование в help --format json.
    ",
    invariants: nonempty_str![
        "Ответ map не превышает 8 КБ независимо от размера проекта.",
        "Ответ state не превышает 2 КБ и содержит только изменяющееся за день.",
        "Усечение вывода обязано быть явным: молчаливое усечение запрещено.",
        "Законный отказ отличается от сбоя полем legitimate.",
    ],
    authors: nonempty_str!["Анищук Сергей"],
    decided_at: dacc_core::date!(2026, 9, 10),
    decided_by: &[],
);
