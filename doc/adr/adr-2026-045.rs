use crate::taxonomy::{channel, Subsystem};
use dacc_core::{nonempty_str, taxon};
use dacc_knowledge::{Breaking, DocStatus};

dacc_knowledge::adr!(
    title: "Инвариант — отдельный тип invariant! со статусом",
    status: DocStatus::Active,
    channel: channel::Public,
    subsystems: &[taxon!(Subsystem, Knowledge), taxon!(Subsystem, Scan)],
    context: r"
        Гарантии, которые обязаны держаться, сейчас записаны прозой в решениях и
        спецификациях, а «обещание» против «проверено» не различимо: реестр не
        знает, кто атаковал инвариант и атаковал ли вообще. Пилот nerpa держит
        локальный тип invariant! со статусом Planned/Claimed/Enforced и списком
        ALL_INVARIANTS руками; dacc эквивалента не поставляет.
    ",
    decision: r"
        Инвариант — отдельный тип invariant! в слое знания (рядом с adr! и
        rfc!): та же ось реестра, тот же скан, та же константа-ссылка
        InvariantRef. Поля: идентификатор, statement (что
        обязано держаться), rationale (почему), enforced_by (разметка кода, где
        инвариант держится) и tests (разметка тестов, которые его доказывают).

        Статус — вариант, а не строка: Planned (решено, не построено), Claimed
        { bypass } (код и тесты есть, обход описан, но не опробован) и Enforced.
        Enforced в свою очередь — Unrepresentable (типы делают нарушение
        невыразимым) или Adversarial { bypass, tests } (обход описан и опробован
        атакующими тестами). Поэтому запись не может сказать «Enforced», не
        сказав, как нарушение было проведено.

        Скан порождает реестр invariant! — модули файлов, ссылки InvariantRef и
        список ALL_INVARIANTS — вместо списка, который пилот держал руками.
        Порядковый номер (i0001, …) порождается сканом, а не пишется в файле:
        выводимое выводится (решение 46).
    ",
    trade_offs: &[
        "Плюс: статус — вариант, поэтому «Enforced» без названного обхода или теста невыразим",
        "Плюс: enforced_by и tests — пути к разметке, удаление которой ломает ссылку",
        "Плюс: скан порождает реестр, а не список руками",
        "Минус: ещё одна ось реестра и ещё один скан-вид",
    ],
    constraints: &[
        "Инвариант — отдельный тип invariant!, а не проза в решении или спецификации.",
        "Статус Enforced требует Unrepresentable или Adversarial с bypass и tests.",
        "enforced_by и tests — ссылки на разметку кода, порождённые сканом.",
        "Скан порождает ALL_INVARIANTS и порядковые номера.",
    ],
    authors: nonempty_str!["Анищук Сергей"],
    decided_at: dacc_core::date!(2026, 9, 30),
    breaking: Breaking::No,
    code_refs: &[],
    related_rfcs: &[crate::rfc::rfc_2026_001],
);
