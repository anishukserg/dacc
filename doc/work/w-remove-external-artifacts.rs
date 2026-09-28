use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use crate::taxonomy::Subsystem;
use dacc_work::WorkOrigin;

dacc_work::work!("w-remove-external-artifacts",
    title: NonEmptyStr::new("Убрать внешние артефакты и каталог spec/"),
    slice: crate::slice::s_commit_rules,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_009),
    taxon: taxon!(Subsystem, Methodology),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "Калитка не находит в дереве имён из локального списка внешних проектов; каталога spec/ и файла ревью нет; ссылки из кода и документов указывают на реестр."
    ),
);
