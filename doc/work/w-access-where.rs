use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-access-where",
    title: NonEmptyStr::new("Место в коде одной командой: cargo dacc where --file"),
    slice: crate::slice::s_access_first_touch,
    origin: WorkOrigin::Specification(crate::rfc::rfc_2026_003),
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new("cargo dacc where --file <путь> называет разметку doc_anchor файла, документы реестра, ссылающиеся на её якоря, и связанные работы с их состояниями; --format json отдаёт то же полями; файл без разметки — отдельный ответ, а не пустой успех."),
);
