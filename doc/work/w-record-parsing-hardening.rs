use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-record-parsing-hardening",
    title: NonEmptyStr::new("Разбор записей не доверяет форме текста"),
    slice: crate::slice::s_agent_robustness,
    origin: WorkOrigin::Divergence {
        specification: crate::rfc::rfc_2026_003,
        limitation: crate::limitation::l_form_not_trusted,
    },
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "strip_comments снимает вложенные блочные комментарии счётом глубины; string_literal читает настоящие escape-последовательности \\n, \\t, \\r и unicode; where и refs ищут связи по тексту без комментариев; запись без поля title показывает statement; каждый пункт доказан тестом."
    ),
);
