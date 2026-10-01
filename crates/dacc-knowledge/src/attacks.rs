//! Атаки на гарантии схемы слоя знания.
//!
//! Каждая атака записана `compile_fail`-тестом с кодом ошибки, и рядом с ней —
//! позитивный контроль той же формы. Без контроля отрицательный тест проходит
//! вакуумно: запись могла не собраться из-за опечатки, а не из-за атакуемого
//! правила. Коды ошибок rustdoc сверяет только в nightly-режиме, поэтому
//! калитка выполняет атаки под `RUSTC_BOOTSTRAP=1` и перед ними пробной атакой
//! проверяет, что сверка работает (решение 12). Без сверки остаётся факт
//! отказа, и от вакуума защищает один контроль.
//!
//! ## E5. Значение чужой оси таксономии там, где ждут подсистему
//!
//! ```compile_fail,E0308
//! dacc_core::declare_taxonomy! { Subsystem => [Storage], Team => [Core] }
//! fn attack(d: &dacc_knowledge::ArchitectureDecision) -> bool {
//!     d.subsystems.contains(&Team::Core)
//! }
//! ```
//!
//! Контроль:
//!
//! ```
//! dacc_core::declare_taxonomy! { Subsystem => [Storage], Team => [Core] }
//! fn control(d: &dacc_knowledge::ArchitectureDecision) -> bool {
//!     d.subsystems.contains(&Subsystem::Storage)
//! }
//! ```
//!
//! ## E6. Пустая строка в непустом списке авторов
//!
//! ```compile_fail,E0308
//! fn attack(d: &mut dacc_knowledge::ArchitectureDecision) {
//!     d.authors = dacc_core::nonempty![""];
//! }
//! ```
//!
//! Контроль:
//!
//! ```
//! fn control(d: &mut dacc_knowledge::ArchitectureDecision) {
//!     d.authors = dacc_core::nonempty_str!["Анищук Сергей"];
//! }
//! ```
//!
//! Пустое значение отвергается при вычислении статика, а не в рантайме:
//!
//! ```compile_fail,E0080
//! static AUTHORS: dacc_core::NonEmpty<dacc_core::NonEmptyStr> =
//!     dacc_core::nonempty_str!["  "];
//! ```

//!
//! Инвариант обязан знать свою спецификацию (работа w-invariant-spec-link):
//! запись без `specification` не собирается — поле обязательное по типу.
//!
//! ```compile_fail,E0063
//! dacc_knowledge::invariant!("i-attack",
//!     status: dacc_knowledge::InvariantStatus::Planned,
//!     statement: dacc_core::NonEmptyStr::new("атака"),
//!     rationale: "без спецификации",
//!     enforced_by: &[],
//!     tests: &[],
//! );
//! ```
//!
//! Позитивный контроль: запись со спецификацией собирается и несёт её.
//!
//! ```
//! static SPEC: dacc_core::RfcRef = dacc_core::RfcRef::__from_scan("rfc-attack");
//! dacc_knowledge::invariant!("i-control",
//!     status: dacc_knowledge::InvariantStatus::Planned,
//!     statement: dacc_core::NonEmptyStr::new("контроль"),
//!     rationale: "спецификация названа",
//!     specification: SPEC,
//!     enforced_by: &[],
//!     tests: &[],
//! );
//! ```
