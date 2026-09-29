use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-readme-badges",
    title: NonEmptyStr::new("README и плашки: версия, покрытие, мутанты"),
    slice: crate::slice::s_github_polish,
    origin: WorkOrigin::Toil {
        justification: NonEmptyStr::new(
            "Публичная страница должна говорить суть ссылками, а не прозой; плашки покрытия и мутаций — это CI, а не код методологии."
        ),
    },
    taxon: taxon!(Subsystem, Methodology),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "README сокращён до сути и ссылок; плашки версии (crates.io), калитки, покрытия (codecov) и мутаций (cargo-mutants); workflows coverage.yml и mutation.yml."
    ),
);
