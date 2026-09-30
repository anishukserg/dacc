//! Общие типы DACC: ссылочные newtype, непустые списки, обратимость.
//!
//! Крейт не содержит ни схемы документов, ни схемы работы — только то, что
//! нужно обоим слоям. Зависимостей нет:
//! крейт подключают и атрибут разметки, и скан, работающий в `build.rs`.

// Нестабильные возможности запрещены и в doctest: lints манифеста на них не
// распространяются, а атаки выполняются под RUSTC_BOOTSTRAP (решение 12).
#![doc(test(attr(forbid(unstable_features))))]

pub mod anchor_rules;
pub mod channel;
pub mod date;
pub mod nonempty;
pub mod radius;
pub mod refs;
pub mod taxonomy;

pub use channel::Channel;
pub use nonempty::{NonEmpty, NonEmptyStr};
pub use radius::{BlastRadius, Severity};
pub use refs::{
    AdrRef, AnchorId, BreakingRef, GateRef, InvariantRef, LimitationRef, ObligationRef, RfcRef,
    SliceRef, SupersededRef, ThrustRef, UpgradeRef, WorkRef,
};
pub use taxonomy::{axis, Taxon};

/// Дата в const-контексте. Несуществующая дата — ошибка компиляции.
///
/// Допустимость даты проверяет `date::is_valid_ymd` — единый валидатор,
/// которым пользуется и разбор времени журнала (`dacc_journal::time`), поэтому
/// «31 февраля» невозможна ни в реестре, ни в журнале. Само значение
/// раскрывается в `::chrono`, поэтому `chrono` обязан быть зависимостью
/// вызывающего крейта; сам `dacc-core` от него не зависит.
#[macro_export]
macro_rules! date {
    ($y:literal, $m:literal, $d:literal) => {{
        const _: () = assert!(
            $crate::date::is_valid_ymd($y, $m, $d),
            "dacc: некорректная дата"
        );
        match ::chrono::NaiveDate::from_ymd_opt($y, $m, $d) {
            Some(d) => d,
            None => panic!("dacc: некорректная дата"),
        }
    }};
}
