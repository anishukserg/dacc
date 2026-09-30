//! Слой знания: что решено и почему. Спецификация — RFC-0001 в документах DACC, `doc/rfc/r0001.rs`.

// Нестабильные возможности запрещены и в doctest: lints манифеста на них не
// распространяются, а атаки выполняются под RUSTC_BOOTSTRAP (решение 12).
#![doc(test(attr(forbid(unstable_features))))]

pub mod attacks;
pub mod schema;
pub use schema::{
    ArchitectureDecision, Breaking, DocStatus, DomainSpecification, Enforced, Invariant,
    InvariantStatus, Review,
};

/// Регистрация решения. Идентификатор — имя файла (`adr-2026-001.rs` для
/// `adr-2026-001`): скан отвергает имя, не являющееся slug, поэтому два
/// решения с одним идентификатором в одном каталоге невыразимы.
#[macro_export]
macro_rules! adr {
    ($($field:ident : $value:expr),* $(,)?) => {
        /// Запись решения этого файла.
        pub static DECISION: $crate::ArchitectureDecision = $crate::ArchitectureDecision {
            $($field: $value),*
        };
    };
}

/// Регистрация доменной спецификации. Идентификатор — имя файла
/// (`rfc-2026-001.rs` для `rfc-2026-001`).
#[macro_export]
macro_rules! rfc {
    ($($field:ident : $value:expr),* $(,)?) => {
        /// Запись спецификации этого файла.
        pub static SPEC: $crate::DomainSpecification = $crate::DomainSpecification {
            $($field: $value),*
        };
    };
}

/// Регистрация инварианта. Файл обязан называться по slug-идентификатору
/// (`i0001.rs`), а slug — первый аргумент. Статус обязателен (решение 45).
#[macro_export]
macro_rules! invariant {
    ($id:literal, $($field:ident : $value:expr),* $(,)?) => {
        /// Запись инварианта этого файла.
        pub static INVARIANT: $crate::Invariant = $crate::Invariant { id: $id, $($field: $value),* };
    };
}

/// Регистрация независимого чтения. Файл обязан называться по
/// slug-идентификатору (`v0001.rs`), а slug — первый аргумент. Угол `angle` и
/// находки `findings` обязательны (решение 46).
#[macro_export]
macro_rules! review {
    ($id:literal, $($field:ident : $value:expr),* $(,)?) => {
        /// Запись чтения этого файла.
        pub static REVIEW: $crate::Review = $crate::Review { id: $id, $($field: $value),* };
    };
}
