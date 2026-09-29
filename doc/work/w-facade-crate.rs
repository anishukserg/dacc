use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-facade-crate",
    title: NonEmptyStr::new("Фасадный крейт — публичная поверхность"),
    slice: crate::slice::s_facade_crate,
    origin: WorkOrigin::Divergence {
        specification: crate::rfc::rfc_2026_001,
        violated: NonEmptyStr::new(
            "Решение 7 называет публичной поверхностью фасадный крейт и cargo-dacc, но фасадного крейта в репозитории нет: потребитель подключает крейты по одному."
        ),
    },
    taxon: taxon!(Subsystem, Knowledge),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Крейт dacc переэкспортирует публичную поверхность dacc-core, dacc-knowledge, dacc-work, dacc-scan, dacc-access и dacc-journal: типы доступны через один крейт, макросы требуют конкретные крейты из-за $crate."
    ),
);
