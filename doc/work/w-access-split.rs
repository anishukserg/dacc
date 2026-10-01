use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-access-split",
    title: NonEmptyStr::new("Разбор access.rs на модули по командам"),
    slice: crate::slice::s_access_modules,
    origin: WorkOrigin::Divergence {
        specification: crate::rfc::rfc_2026_003,
        limitation: crate::limitation::l_two_parsers,
    },
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "access.rs становится каталогом access/ по командам слоя доступа; запись реестра читается значениями полей в одном месте, разбор аргументов и форма строк JSON и XML — в одном экземпляре; комментарий не подделывает поле записи в ответах слоя доступа, формы ответов и коды отказов не меняются."
    ),
);
