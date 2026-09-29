use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-pages-manual-enable",
    title: NonEmptyStr::new("Убрать enablement и включить Pages вручную"),
    slice: crate::slice::s_pages_manual,
    origin: WorkOrigin::Toil {
        justification: NonEmptyStr::new(
            "enablement: true падает с «Resource not accessible by integration»: интеграция не может включить Pages; включение — вручную в настройках репозитория."
        ),
    },
    taxon: taxon!(Subsystem, Access),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "pages.yml без enablement: true; Pages включается вручную — Settings → Pages → Source: GitHub Actions."
    ),
);
