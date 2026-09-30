use crate::taxonomy::{channel, Subsystem};
use dacc_core::{nonempty_str, taxon};
use dacc_knowledge::{Breaking, DocStatus};

dacc_knowledge::adr!(
    title: "Независимое чтение — отдельный тип review!",
    status: DocStatus::Active,
    channel: channel::Public,
    subsystems: &[taxon!(Subsystem, Knowledge), taxon!(Subsystem, Scan)],
    context: r"
        Независимое чтение вердикта сейчас — проза: угол чтения, находки и то, что
        с каждой стало, не являются типизированными записями, поэтому их нельзя
        перечислить и сверить. Пилот nerpa держит локальный тип review! с angle,
        findings, authors и decided_at, а список ALL_REVIEWS — руками; dacc
        эквивалента не поставляет.
    ",
    decision: r"
        Независимое чтение — отдельный тип review! в слое знания (рядом с adr! и
        rfc!): та же ось реестра, тот же скан, та же константа-ссылка ReviewRef.
        Поля: title (что прочитано),
        angle (угол, под которым читали, — граница чтения на записи), findings
        (каждая находка и что с ней стало, непустым списком), authors (кто читал;
        для вердикта — читатель, не пишущий других записей) и decided_at.

        angle обязателен: чтение без названного угла не называет свою границу и
        выдаёт частное за полное. Скан порождает реестр review! — модули файлов,
        ссылки ReviewRef и список ALL_REVIEWS — вместо списка руками; порядковый
        номер (v0001, …) порождается сканом (решение 46).
    ",
    trade_offs: &[
        "Плюс: угол чтения — поле, а не проза, поэтому граница чтения на записи",
        "Плюс: findings — непустой список, поэтому чтение без находки невыразимо",
        "Плюс: скан порождает реестр, а не список руками",
        "Минус: ещё одна ось реестра",
    ],
    constraints: &[
        "Чтение — отдельный тип review!, а не проза.",
        "Угол чтения angle обязателен и непуст.",
        "Находки findings — непустой список.",
        "Скан порождает ALL_REVIEWS и порядковые номера.",
    ],
    authors: nonempty_str!["Анищук Сергей"],
    decided_at: dacc_core::date!(2026, 9, 30),
    breaking: Breaking::No,
    code_refs: &[],
    related_rfcs: &[crate::rfc::rfc_2026_001],
);
