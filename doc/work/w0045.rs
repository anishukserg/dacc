use crate::taxonomy::Subsystem;
use slipway_core::{taxon, BlastRadius, NonEmptyStr};
use slipway_work::WorkOrigin;

slipway_work::work!(45,
    title: NonEmptyStr::new("Отказ настройки называет форму числа; диапазон покрывает корневой коммит"),
    slice: crate::slice::s0011,
    origin: WorkOrigin::Divergence {
        specification: crate::rfc::r0002,
        violated: NonEmptyStr::new(
            "Отказ называет, как его исправить, а проверка оснований покрывает каждый коммит на пути в основную ветку, включая корневой (решения 8 и 18)."
        ),
    },
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Local,
    outcome: NonEmptyStr::new(
        "Отказ по числовому ключу настройки показывает годную запись в кавычках; проверка диапазона покрывает безродительский коммит, и CI берёт для новой ветки диапазон самого коммита."
    ),
);
