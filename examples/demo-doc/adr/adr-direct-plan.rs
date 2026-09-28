//! Действующее решение.
use crate::taxonomy::Subsystem;
use dacc_core::{nonempty_str, taxon};
use dacc_knowledge::{Breaking, DocStatus};

dacc_knowledge::adr!(
    title: "Прямая передача плана из ORM в исполнитель",
    status: DocStatus::Active,
    subsystems: &[taxon!(Subsystem, Executor), taxon!(Subsystem, Storage)],
    context: r"
        Короткие транзакции тратят заметную долю времени на разбор SQL.
    ",
    decision: r"
        Генерировать бинарный план на стороне ORM и передавать его напрямую.
    ",
    trade_offs: &[
        "Плюс: нет разбора текста на горячем пути",
        "Минус: нужно стабилизировать двоичное представление плана",
    ],
    constraints: &["Запрещён текстовый SQL на пути прямого исполнителя."],
    authors: nonempty_str!["Анищук Сергей"],
    decided_at: dacc_core::date!(2026, 9, 8),
    breaking: Breaking::No,
    code_refs: &[crate::anchor::plan_ir],
    related_rfcs: &[],
);
