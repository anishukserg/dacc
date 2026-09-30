use crate::taxonomy::{channel, Subsystem};
use dacc_core::{nonempty_str, taxon};
use dacc_knowledge::{Breaking, DocStatus};

dacc_knowledge::adr!(
    title: "Инструкция повышения версии — отдельная запись upgrade!",
    status: DocStatus::Active,
    channel: channel::Public,
    subsystems: &[taxon!(Subsystem, Work), taxon!(Subsystem, Knowledge)],
    context: r"
        Потребитель при повышении версии dacc не знает, что менять: ломающие
        изменения описаны прозой в CHANGELOG, а не типом, и инструкция не
        порождается и не выводится командой. CHANGELOG пишется руками и
        расходится с реестром. Поле Breaking в ADR покрывает миграцию
        собственного реестра dacc, а не шаги потребителя.
    ",
    decision: r"
        Инструкция повышения версии — отдельная запись реестра upgrade! с полями:
        идентификатор, from и to (версии до и после), subject (что изменилось) и
        how (что потребитель обязан сделать). Все четыре текстовых поля
        обязательны и непусты по типу: шаг без способа исправления невыразим.

        CHANGELOG порождается из реестра upgrade!, а не пишется руками:
        команда cargo dacc upgrade <from> <to> выводит шаги subject + how между
        версиями, а без аргументов — все шаги, сгруппированные по версии. Секция
        «Upgrade notes» CHANGELOG.md — вывод этой команды, закоммиченный, как
        статичный сайт.

        Запись upgrade! живёт в слое работы рядом с obligation! и limitation!:
        та же ось реестра, тот же скан, та же константа-ссылка UpgradeRef.
        Версии — строки, а не semver-тип: сравнение версий остаётся текстовым
        совпадением, а не арифметикой, которую пока некому потреблять.
    ",
    trade_offs: &[
        "Плюс: шаг повышения версии — типизированная запись, а не проза; опечатка в реестре не компилируется",
        "Плюс: from/to/subject/how обязательны и непусты — шаг без способа исправления невыразим",
        "Плюс: CHANGELOG порождается командой из реестра и не расходится с ним",
        "Минус: ещё одна ось реестра и ещё одна команда",
    ],
    constraints: &[
        "Инструкция — отдельная запись upgrade!, а не проза в CHANGELOG.",
        "Поля from/to/subject/how обязательны и непусты.",
        "cargo dacc upgrade <from> <to> выводит шаги subject + how; без аргументов — все шаги по версиям.",
        "Секция «Upgrade notes» CHANGELOG.md — вывод команды, закоммиченный.",
    ],
    authors: nonempty_str!["Анищук Сергей"],
    decided_at: dacc_core::date!(2026, 9, 30),
    breaking: Breaking::No,
    code_refs: &[],
    related_rfcs: &[crate::rfc::rfc_2026_002],
);
