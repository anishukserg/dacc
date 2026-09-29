//! Замещённое решение: ссылка на замещающее — путь к константе, а не число.
use crate::taxonomy::{channel, Subsystem};
use dacc_core::{nonempty_str, taxon};
use dacc_knowledge::{Breaking, DocStatus};

dacc_knowledge::adr!(
    title: "Текстовый SQL как единственный путь исполнения",
    status: DocStatus::SupersededBy(crate::adr::adr_direct_plan),
    channel: channel::Public,
    subsystems: &[taxon!(Subsystem, Executor)],
    context: r"
        Исходное устройство: любой запрос проходит через разбор текста SQL.
    ",
    decision: r"
        Единственный путь исполнения — текстовый SQL.
    ",
    trade_offs: &["Плюс: простота", "Минус: разбор текста на горячем пути"],
    constraints: &[],
    authors: nonempty_str!["Анищук Сергей"],
    decided_at: dacc_core::date!(2026, 3, 1),
    breaking: Breaking::No,
    code_refs: &[],
    related_rfcs: &[],
);
