//! Канал публикации документа (решение 21): продукт объявляет значения, а
//! `channel!` резолвится путём к константе — опечатка становится ошибкой
//! компилятора, а не тихим каналом. Спецификация — RFC-0003.

use std::{fmt, hash};

/// Канал публикации документа. Один канал на документ: ссылка через границу
/// канала — отказ экспорта (решение 21).
#[derive(Clone, Copy)]
pub struct Channel(&'static str);

impl Channel {
    /// Только для `declare_channels!`. В тексте реестра вызов отвергается сканом.
    #[doc(hidden)]
    pub const fn __new_unchecked(name: &'static str) -> Self {
        Self(name)
    }

    pub const fn as_str(&self) -> &'static str {
        self.0
    }
}

impl PartialEq for Channel {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Channel {}

impl hash::Hash for Channel {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl fmt::Debug for Channel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Channel").field(&self.0).finish()
    }
}

/// Объявление каналов публикации продукта. Порождает модуль `channel` с
/// константами, поэтому `channel::Public` при опечатке даёт `unresolved path`,
/// а не проходит молча.
#[macro_export]
macro_rules! declare_channels {
    ($($value:ident),* $(,)?) => {
        #[doc = "Каналы публикации продукта."]
        #[allow(non_upper_case_globals, non_snake_case)]
        pub mod channel {
            $(
                #[doc = concat!("Канал `", stringify!($value), "`.")]
                pub const $value: $crate::Channel = $crate::Channel::__new_unchecked(stringify!($value));
            )*
            /// Все объявленные каналы в порядке объявления.
            pub const ALL: &[$crate::Channel] = &[$($value),*];
        }
    };
}
