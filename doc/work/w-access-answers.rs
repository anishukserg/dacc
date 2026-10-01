use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-access-answers",
    title: NonEmptyStr::new("Ответы инструмента говорят факт, а не подстроку"),
    slice: crate::slice::s_access_modules,
    origin: WorkOrigin::Divergence {
        specification: crate::rfc::rfc_2026_003,
        limitation: crate::limitation::l_access_answers,
    },
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Счётчики карты ведут каждую стадию, включая landed from history; ошибка аргумента называет аргумент; поле truncated перечня соответствует усечению с shown и total; разметка where читается из кода без комментариев; метрика считает коммиты, а не строки трейлеров; каждый пункт доказан тестом, и ответы слоя доступа несут инвариант i-access-answers."
    ),
);
