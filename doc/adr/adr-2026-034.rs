use crate::taxonomy::{channel, Subsystem};
use dacc_core::{nonempty_str, taxon};
use dacc_knowledge::{Breaking, DocStatus};

dacc_knowledge::adr!(
    title: "Набор публикации",
    status: DocStatus::Active,
    channel: channel::Public,
    subsystems: &[taxon!(Subsystem, Methodology), taxon!(Subsystem, Knowledge), taxon!(Subsystem, Cli)],
    context: r"
        Решение 25 закрепило публикацию пяти крейтов L1: core, journal,
        knowledge, derive, scan. С тех пор выросли слой L2 (dacc-work,
        dacc-access) и инструмент dacc-cli; все несут поля публикации
        (решение 17) и делят версию рабочего пространства. Какой набор
        публикуется теперь, и в каком порядке?
    ",
    decision: r"
        Публикуется полный набор из восьми крейтов: пять L1 (core, journal,
        knowledge, derive, scan) плюс слой L2 (work, access) и инструмент cli.
        Порядок — по зависимостям: core и journal без зависимостей; knowledge,
        derive, work на core; scan на core и journal; access на core, knowledge,
        work; cli на journal. doc/ и examples/ не публикуются (решение 25).
        Фасадный крейт из решения 7 — отдельная работа, не часть этого выпуска.
    ",
    trade_offs: &[
        "Плюс: потребитель получает весь слой L2 и инструмент, а не только ядро L1",
        "Плюс: порядок по зависимостям воспроизводим и проверяем при публикации",
        "Минус: восемь крейтов — восемь пакетов; единой точки входа пока нет (фасадный крейт — отдельно)",
    ],
    constraints: &[
        "Публикуются восемь крейтов; doc/ и examples/ не публикуются.",
        "Порядок публикации — по зависимостям, единой версией рабочего пространства.",
        "Фасадный крейт не входит в этот набор — его вводит отдельное решение.",
    ],
    authors: nonempty_str!["Анищук Сергей"],
    decided_at: dacc_core::date!(2026, 9, 29),
    breaking: Breaking::No,
    code_refs: &[],
    related_rfcs: &[crate::rfc::rfc_2026_001],
);
