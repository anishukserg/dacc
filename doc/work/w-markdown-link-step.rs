use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use crate::taxonomy::Subsystem;
use dacc_work::WorkOrigin;

dacc_work::work!("w-markdown-link-step",
    title: NonEmptyStr::new("Шаг ссылок в markdown не выполнялся на дереве коммита"),
    slice: crate::slice::s_commit_rules,
    origin: WorkOrigin::Divergence {
        specification: crate::rfc::rfc_2026_002,
        limitation: crate::limitation::l_markdown_link_step,
    },
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "Самотест, падавший до починки, проходит: битая ссылка в markdown отвергается и в дереве внутри каталога git; калитка на дереве коммита выполняет все восемь шагов."
    ),
);
