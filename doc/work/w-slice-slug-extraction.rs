use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-slice-slug-extraction",
    title: NonEmptyStr::new("Record::parse извлекает slug среза, а не идентификатор"),
    slice: crate::slice::s_slug_fixes,
    origin: WorkOrigin::Toil {
        justification: NonEmptyStr::new(
            "Record::parse читал slice: crate::slice::s_work_layer_slugs и отдавал s_work_layer_slugs (идентификатор), а Subject::Slice несёт slug s-work-layer-slugs; slice close не находил работы среза."
        ),
    },
    taxon: taxon!(Subsystem, Work),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "Record::parse переводит подчёркивания идентификатора в дефисы slug; slice close s-work-layer-slugs находит свои работы."
    ),
);
