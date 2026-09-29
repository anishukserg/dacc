use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-pages-enablement",
    title: NonEmptyStr::new("Включить Pages через enablement в pages.yml"),
    slice: crate::slice::s_pages_ci,
    origin: WorkOrigin::Toil {
        justification: NonEmptyStr::new(
            "actions/configure-pages падает с «Get Pages site failed», когда Pages не включена; enablement: true включает её через API."
        ),
    },
    taxon: taxon!(Subsystem, Access),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "pages.yml передаёт enablement: true в configure-pages; публикация не падает на «Get Pages site failed»."
    ),
);
