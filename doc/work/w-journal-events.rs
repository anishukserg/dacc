use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-journal-events",
    title: NonEmptyStr::new("Журнал событий: формат, автомат переходов, свёртка при сборке реестра"),
    slice: crate::slice::s_work_journal,
    origin: WorkOrigin::Specification(crate::rfc::rfc_2026_002),
    taxon: taxon!(Subsystem, Work),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Состояние каждой единицы работы — свёртка каталога doc/journal при сборке крейта документов; статусного поля нет; недопустимый переход не собирается и называет файл события; событие о работе вне плана не разрешается; незавершённая работа в закрытом срезе не собирается."
    ),
);
