use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-access-refs-find",
    title: NonEmptyStr::new("Граф и поиск: cargo dacc refs и find"),
    slice: crate::slice::s_access_contract,
    origin: WorkOrigin::Specification(crate::rfc::rfc_2026_003),
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new("cargo dacc refs <id> называет исходящие ссылки записи и записи, ссылающиеся на неё; cargo dacc find <текст> ищет по идентификаторам и заголовкам реестра; ответы текстом и json; отсутствие совпадений — отдельный ответ, а не пустой успех."),
);
