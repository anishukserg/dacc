use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use crate::taxonomy::Subsystem;
use dacc_work::WorkOrigin;

dacc_work::work!("w-commit-rules-check",
    title: NonEmptyStr::new("Правила коммитов: проверка сообщения, калитка на дереве коммита, обёртка коммита"),
    slice: crate::slice::s_commit_rules,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_008),
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "Самотест tools/test-commit-rules.sh проходит: неверная форма, отсутствующее или несуществующее основание, пустой набор путей и внешнее имя отвергаются, корректный коммит принимается ровно с перечисленными путями."
    ),
);
