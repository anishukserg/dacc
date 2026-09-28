use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-docs-single-crate",
    title: NonEmptyStr::new("Документы DACC в одном крейте doc/"),
    slice: crate::slice::s_single_doc_crate,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_011),
    taxon: taxon!(Subsystem, Knowledge),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Крейты dacc-meta и dacc-plan заменены крейтом dacc-doc в doc/, демо-реестр переименован в demo-doc; документы ссылаются друг на друга через crate::; калитка и самотест правил коммитов проходят по новым путям."
    ),
);
