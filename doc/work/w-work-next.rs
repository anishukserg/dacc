use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-work-next",
    title: NonEmptyStr::new("work next: допуск задачи и WIP-лимит под локом журнала"),
    slice: crate::slice::s_agent_task_model,
    origin: WorkOrigin::Specification(crate::rfc::rfc_2026_002),
    taxon: taxon!(Subsystem, Work),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "cargo dacc work next берёт следующую запланированную работу открытого среза в порядке плана и пишет started под межпроцессным локом журнала; WIP-лимит из настройки wip_limit отказывает переполнению кодом wip-limit и называет занятые работы, а пустой допуск — кодом nothing-to-take; приём и отказы покрыты тестами."
    ),
);
