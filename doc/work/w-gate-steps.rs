use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use crate::taxonomy::Subsystem;
use dacc_work::WorkOrigin;

dacc_work::work!("w-gate-steps",
    title: NonEmptyStr::new("Калитка: форматирование, clippy, документация, скрипты, ссылки, защита публикации"),
    slice: crate::slice::s_commit_rules,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_008),
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "Калитка на дереве коммита проходит восемь шагов с тулчейном из rust-toolchain.toml; самотест подтверждает, что pre-push отвергает историю со следом внешнего имени и ветки архива и пропускает чистую историю."
    ),
);
