use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-scan-english-errors",
    title: NonEmptyStr::new("Порождённый сканом текст и его ошибки — на английском"),
    slice: crate::slice::s_scan_english,
    origin: WorkOrigin::Divergence {
        specification: crate::rfc::rfc_2026_001,
        violated: NonEmptyStr::new(
            "Текст, порождаемый инструментом, английский, включая комментарии и документацию порождённых файлов и сообщения, которые они выдают компилятором (решение 22)."
        ),
    },
    taxon: taxon!(Subsystem, Scan),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Шапка, документация модулей, констант и списков в порождённом коде английские; английские и сообщения об ошибках скана и порождаемые compile_error и константные проверки; страницы документации чужого реестра не смешивают языки; сценарий закрепляет снимок порождённого."
    ),
);
