//! Ссылочные типы. Ключевой механизм методологии: ссылка между документами —
//! не число и не строка, а **путь к константе**, порождённой сканом реестра.
//!
//! Следствие: опечатка, удаление документа или ссылка не того класса дают
//! `unresolved path` — обычную ошибку компилятора с подсказкой похожего имени.
//! Приём из решений 1 и 4.
//!
//! Поле каждой ссылки закрыто, значение порождает скан через `__from_scan`.
//! Запечатать конструктор полностью Rust не позволяет: константа порождается
//! в чужом крейте и обязана быть там конструируемой. Поэтому ручной вызов
//! `__from_scan` в тексте реестра отвергает сам скан (`dacc-scan`, сила
//! `BuildScript`); вне файлов реестра обход остаётся выразимым.
//!
//! Виды ссылок не смешиваются: у одних внутри число (слой работы), у других —
//! slug (слой знания):
//!
//! ```compile_fail,E0308
//! let gate = dacc_core::GateRef::__from_scan(2);
//! let _: dacc_core::AdrRef = gate;
//! ```
//!
//! Контроль:
//!
//! ```
//! let gate = dacc_core::GateRef::__from_scan(2);
//! let _: dacc_core::GateRef = gate;
//! ```

macro_rules! declare_ref {
    ($(#[$m:meta])* $name:ident($repr:ty), $example:literal) => {
        $(#[$m])*
        ///
        /// Атака E1 — подделка ссылки конструктором — не собирается:
        ///
        #[doc = concat!("```compile_fail,E0423\nlet _forged = dacc_core::", stringify!($name), "(", $example, ");\n```")]
        ///
        /// Позитивный контроль: тот же путь собирается через конструктор
        /// скана, иначе отрицательный тест выше мог бы пройти вакуумно.
        ///
        #[doc = concat!("```\nlet _ok = dacc_core::", stringify!($name), "::__from_scan(", $example, ");\n```")]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name($repr);

        impl $name {
            /// Только для кода, порождённого сканом. В тексте реестра вызов
            /// отвергается сканом как обход невыразимости.
            #[doc(hidden)]
            pub const fn __from_scan(value: $repr) -> Self {
                Self(value)
            }
        }
    };
}

declare_ref! {
    /// Ссылка на архитектурное решение. Константы порождаются сканом
    /// реестра решений; отсутствующее решение — не резолвится. Несёт slug
    /// из имени файла решения, а не порядковый номер.
    AdrRef(&'static str), "\"adr-2026-001\""
}

declare_ref! {
    /// Ссылка на доменную спецификацию. Несёт slug из имени файла.
    RfcRef(&'static str), "\"rfc-2026-001\""
}

declare_ref! {
    /// Ссылка на решение **в статусе «замещено»**. Отдельный тип, потому что
    /// константы порождаются только для замещённых решений: «удаление живого
    /// кода под видом уборки» невыразимо.
    SupersededRef(&'static str), "\"adr-2026-001\""
}

declare_ref! {
    /// Ссылка на **ломающее** решение. Константы — только для решений
    /// с инструкциями миграции.
    BreakingRef(&'static str), "\"adr-2026-001\""
}

declare_ref! {
    /// Ссылка на разметку кода (`#[doc_anchor]`). Константы порождаются
    /// сканом исходников: удалили разметку — ссылка не собирается.
    ///
    /// Несёт идентификатор разметки, а не порядковый номер: номер сдвигался
    /// бы при каждой новой разметке и устаревал в журнале и индексе.
    AnchorId(&'static str), "\"plan-ir\""
}

declare_ref! {
    /// Ссылка на гейт каталога.
    GateRef(u32), "2"
}

declare_ref! {
    /// Ссылка на направление (Thrust).
    ThrustRef(u32), "2"
}

declare_ref! {
    /// Ссылка на срез (Slice).
    SliceRef(u32), "2"
}

declare_ref! {
    /// Ссылка на единицу работы (WorkItem).
    WorkRef(u32), "2"
}

macro_rules! numbered {
    ($($name:ident),*) => {$(
        impl $name {
            pub const fn index(self) -> u32 {
                self.0
            }
        }
    )*};
}

numbered!(GateRef, ThrustRef, SliceRef, WorkRef);

macro_rules! slug_ref {
    ($($name:ident),*) => {$(
        impl $name {
            /// Идентификатор документа: slug из имени файла.
            pub const fn as_str(self) -> &'static str {
                self.0
            }
        }
    )*};
}

slug_ref!(AdrRef, RfcRef, SupersededRef, BreakingRef);

impl AnchorId {
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbered_refs_keep_their_index() {
        assert_eq!(GateRef::__from_scan(7).index(), 7);
    }

    #[test]
    fn slug_refs_keep_their_id() {
        assert_eq!(AdrRef::__from_scan("adr-2026-001").as_str(), "adr-2026-001");
    }

    #[test]
    fn anchor_is_addressed_by_stable_id() {
        assert_eq!(AnchorId::__from_scan("plan-ir").as_str(), "plan-ir");
    }
}
