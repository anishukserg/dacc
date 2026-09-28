use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-cargo-deny",
    title: NonEmptyStr::new("Проверка зависимостей cargo-deny"),
    slice: crate::slice::s_stable_build,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_013),
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "deny.toml записывает политику зависимостей; калитка коммита проверяет её по сохранённой базе без сети, pre-push — по свежей базе на выгруженной публикуемой вершине; крейты документов и демо не публикуются, зависимости между библиотеками несут версию."
    ),
);
