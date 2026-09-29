use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-site-theme",
    title: NonEmptyStr::new("Светлая тема и схлопываемые разделы сайта"),
    slice: crate::slice::s_site_theme,
    origin: WorkOrigin::Toil {
        justification: NonEmptyStr::new(
            "Стандартная тёмная тема mdBook не нравится, разделы решений и спецификаций слишком длинны: нужны светлая тема и схлопывание."
        ),
    },
    taxon: taxon!(Subsystem, Access),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "Сайт реестра: светлая тема dioxus (Inter, фиолетовый акцент), второстепенные разделы — в <details> по умолчанию закрыты, интерфейс mdBook на русском."
    ),
);
