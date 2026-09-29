//! Таксономия продукта. Опечатка в значении — ошибка компилятора.
dacc_core::declare_channels! { Public }

dacc_core::declare_taxonomy! {
    Subsystem => [Executor, Storage, Import],
    Team => [CoreDb, Platform],
}
