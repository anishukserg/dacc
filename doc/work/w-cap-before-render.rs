use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-cap-before-render",
    title: NonEmptyStr::new("Усечение ответов до сериализации"),
    slice: crate::slice::s_gate_integrity,
    origin: WorkOrigin::Divergence { specification: crate::rfc::rfc_2026_002, limitation: crate::limitation::l_gate_bypass },
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new("map, state и xml собираются из уже усечённых записей: бюджет по элементам, поле truncated штатно, порядок ключей не участвует; тест с названием, содержащим фигурные скобки и перевод строки, доказывает валидность json после усечения."),
);
