use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-stable-toolchain",
    title: NonEmptyStr::new("Stable-тулчейн и сверка кодов ошибок атак"),
    slice: crate::slice::s_stable_build,
    origin: WorkOrigin::Decision(crate::adr::adr_2026_012),
    taxon: taxon!(Subsystem, Cli),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "rust-toolchain.toml закрепляет stable 1.98.1; калитка выполняет атаки под RUSTC_BOOTSTRAP в отдельном каталоге сборки после пробной атаки, которая падает с неверным кодом и проходит с верным; #![feature] запрещён в крейтах и в их doctest."
    ),
);
