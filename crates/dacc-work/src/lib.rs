//! Слой работы: направления, срезы, единицы работы. Спецификация — RFC-0002 в документах DACC, `doc/rfc/r0002.rs`.
//!
//! Минимальный срез для самообкатки: только поля, без которых ломается
//! инвариант «работа имеет основание». Статуса у записей нет — по инварианту 3
//! состояние работы есть свёртка журнала: `WorkState` порождается при сборке
//! реестра из событий (решение 15).
//!
//! Чего нет и почему: `severity`, `service_class`, `size`, `depends_on`,
//! `touches`, ссылок на гейты. Они появятся вместе с механизмами, которые их
//! потребляют: каталогом гейтов, потоком, границами задачи. Поле без
//! потребителя — налог на заведение работы без выигрыша.

// Нестабильные возможности запрещены и в doctest: lints манифеста на них не
// распространяются, а атаки выполняются под RUSTC_BOOTSTRAP (решение 12).
#![doc(test(attr(forbid(unstable_features))))]

pub mod attacks;
pub mod roadmap;
mod schema;

pub use schema::{
    limitations_declared, obligations_redeemed, radius_within_slice, toil_within_local_radius,
    Enforced, InquiryOutcome, Invariant, InvariantStatus, Limitation, Obligation, Review, Slice,
    Thrust, Upgrade, WorkItem, WorkOrigin, WorkState,
};

/// Регистрация направления. Файл обязан называться по slug-идентификатору
/// (`t-pilot.rs`), а slug — первый аргумент.
#[macro_export]
macro_rules! thrust {
    ($id:literal, $($field:ident : $value:expr),* $(,)?) => {
        /// Запись направления этого файла.
        pub static THRUST: $crate::Thrust = $crate::Thrust { id: $id, $($field: $value),* };
    };
}

/// Регистрация среза. Файл обязан называться по slug-идентификатору
/// (`s-versioning.rs`), а slug — первый аргумент.
#[macro_export]
macro_rules! slice {
    ($id:literal, $($field:ident : $value:expr),* $(,)?) => {
        /// Запись среза этого файла.
        pub static SLICE: $crate::Slice = $crate::Slice { id: $id, $($field: $value),* };
    };
}

/// Регистрация единицы работы. Файл обязан называться по slug-идентификатору
/// (`w-fix-gitignore.rs`), а slug — первый аргумент.
#[macro_export]
macro_rules! work {
    ($id:literal, $($field:ident : $value:expr),* $(,)?) => {
        /// Запись единицы работы этого файла.
        pub static WORK: $crate::WorkItem = $crate::WorkItem { id: $id, $($field: $value),* };
    };
}

/// Регистрация обязательства. Файл обязан называться по slug-идентификатору
/// (`o-fix-x.rs`), а slug — первый аргумент. Условие погашения
/// `discharged_when` и критерии решения `criteria` обязательны (решение 35):
/// вопрос не погашается «когда-нибудь» без названной меры.
#[macro_export]
macro_rules! obligation {
    ($id:literal, $($field:ident : $value:expr),* $(,)?) => {
        /// Запись обязательства этого файла.
        pub static OBLIGATION: $crate::Obligation = $crate::Obligation { id: $id, $($field: $value),* };
    };
}

/// Регистрация ограничения. Файл обязан называться по slug-идентификатору
/// (`l-fix-x.rs`), а slug — первый аргумент. Утверждение о расхождении
/// `violated` обязательно: необъявленное расхождение не выразимо.
#[macro_export]
macro_rules! limitation {
    ($id:literal, $($field:ident : $value:expr),* $(,)?) => {
        /// Запись ограничения этого файла.
        pub static LIMITATION: $crate::Limitation = $crate::Limitation { id: $id, $($field: $value),* };
    };
}

/// Регистрация инструкции повышения версии. Файл обязан называться по
/// slug-идентификатору (`u-0-4-0-scan-journal.rs`), а slug — первый аргумент.
/// Поля `from`, `to`, `subject` и `how` обязательны (решение 44).
#[macro_export]
macro_rules! upgrade {
    ($id:literal, $($field:ident : $value:expr),* $(,)?) => {
        /// Запись инструкции повышения версии этого файла.
        pub static UPGRADE: $crate::Upgrade = $crate::Upgrade { id: $id, $($field: $value),* };
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
