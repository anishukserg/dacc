use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-shorten-titles",
    title: NonEmptyStr::new("Короткие названия решений и спецификаций"),
    slice: crate::slice::s_site_polish,
    origin: WorkOrigin::Toil {
        justification: NonEmptyStr::new(
            "Названия ADR/RFC — развёрнутые предложения, боковая панель mdBook становится громоздкой; короткая тема делает её компактной."
        ),
    },
    taxon: taxon!(Subsystem, Knowledge),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "Названия решений и спецификаций сокращены до короткой темы; боковая панель сайта читается без горизонтальной прокрутки."
    ),
);
