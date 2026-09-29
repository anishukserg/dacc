//! Атаки на гарантии схемы слоя работы. Каждая — `compile_fail`-тест
//! с кодом ошибки и позитивным контролем той же формы.
//!
//! ## Исследование, объявляющее приземление кода
//!
//! ```compile_fail,E0559
//! use dacc_core::nonempty_str;
//! fn attack() -> dacc_work::WorkOrigin {
//!     dacc_work::WorkOrigin::Inquiry {
//!         question: *nonempty_str!["Нужен ли CI?"].first(),
//!         produces: dacc_work::InquiryOutcome::Adr,
//!         timebox_days: std::num::NonZeroU16::new(5).unwrap(),
//!         lands_code: true,
//!     }
//! }
//! ```
//!
//! Контроль:
//!
//! ```
//! use dacc_core::nonempty_str;
//! fn control() -> dacc_work::WorkOrigin {
//!     dacc_work::WorkOrigin::Inquiry {
//!         question: *nonempty_str!["Нужен ли CI?"].first(),
//!         produces: dacc_work::InquiryOutcome::Adr,
//!         timebox_days: std::num::NonZeroU16::new(5).unwrap(),
//!     }
//! }
//! assert!(!control().lands_code());
//! ```
//!
//! ## Исследование без срока
//!
//! ```compile_fail,E0080
//! static TIMEBOX: std::num::NonZeroU16 = std::num::NonZeroU16::new(0).unwrap();
//! ```
//!
//! ## Радиус работы выше потолка её среза
//!
//! ```compile_fail,E0080
//! use dacc_core::{BlastRadius, NonEmptyStr, RfcRef, SliceRef, ThrustRef};
//! use dacc_work::{Slice, WorkItem, WorkOrigin};
//! dacc_core::declare_taxonomy! { Subsystem => [Core] }
//! static SLICE: Slice = Slice {
//!     id: "s-001", title: NonEmptyStr::new("срез"), thrust: ThrustRef::__from_scan("t-001"),
//!     outcome: NonEmptyStr::new("исход"), specification: RfcRef::__from_scan("rfc-2026-001"),
//!     max_radius: BlastRadius::Crate,
//! };
//! static WORK: WorkItem = WorkItem {
//!     id: "w-001", title: NonEmptyStr::new("работа"), slice: SliceRef::__from_scan("s-001"),
//!     origin: WorkOrigin::Toil { justification: NonEmptyStr::new("рутина") },
//!     taxon: Subsystem::Core, radius: BlastRadius::Persistent,
//!     outcome: NonEmptyStr::new("готово"),
//! };
//! const _: () = assert!(dacc_work::radius_within_slice(&WORK, &[&SLICE]));
//! ```
//!
//! Контроль — тот же срез, радиус в пределах потолка:
//!
//! ```
//! use dacc_core::{BlastRadius, NonEmptyStr, RfcRef, SliceRef, ThrustRef};
//! use dacc_work::{Slice, WorkItem, WorkOrigin};
//! dacc_core::declare_taxonomy! { Subsystem => [Core] }
//! static SLICE: Slice = Slice {
//!     id: "s-001", title: NonEmptyStr::new("срез"), thrust: ThrustRef::__from_scan("t-001"),
//!     outcome: NonEmptyStr::new("исход"), specification: RfcRef::__from_scan("rfc-2026-001"),
//!     max_radius: BlastRadius::Crate,
//! };
//! static WORK: WorkItem = WorkItem {
//!     id: "w-001", title: NonEmptyStr::new("работа"), slice: SliceRef::__from_scan("s-001"),
//!     origin: WorkOrigin::Toil { justification: NonEmptyStr::new("рутина") },
//!     taxon: Subsystem::Core, radius: BlastRadius::Local,
//!     outcome: NonEmptyStr::new("готово"),
//! };
//! const _: () = assert!(dacc_work::radius_within_slice(&WORK, &[&SLICE]));
//! ```
//!
//! ## Уборка по ссылке на живое решение
//!
//! ```compile_fail,E0308
//! fn attack(live: dacc_core::AdrRef) -> dacc_work::WorkOrigin {
//!     dacc_work::WorkOrigin::Retirement(live)
//! }
//! ```
//!
//! Контроль:
//!
//! ```
//! fn control(retired: dacc_core::SupersededRef) -> dacc_work::WorkOrigin {
//!     dacc_work::WorkOrigin::Retirement(retired)
//! }
//! ```
//!
//! ## Рутина объявляет радиус выше Local
//!
//! ```compile_fail,E0080
//! use dacc_core::{BlastRadius, NonEmptyStr, RfcRef, SliceRef, ThrustRef};
//! use dacc_work::{Slice, WorkItem, WorkOrigin};
//! dacc_core::declare_taxonomy! { Subsystem => [Core] }
//! static SLICE: Slice = Slice {
//!     id: "s-001", title: NonEmptyStr::new("срез"), thrust: ThrustRef::__from_scan("t-001"),
//!     outcome: NonEmptyStr::new("исход"), specification: RfcRef::__from_scan("rfc-2026-001"),
//!     max_radius: BlastRadius::Crate,
//! };
//! static WORK: WorkItem = WorkItem {
//!     id: "w-001", title: NonEmptyStr::new("работа"), slice: SliceRef::__from_scan("s-001"),
//!     origin: WorkOrigin::Toil { justification: NonEmptyStr::new("рутина") },
//!     taxon: Subsystem::Core, radius: BlastRadius::Crate,
//!     outcome: NonEmptyStr::new("готово"),
//! };
//! const _: () = assert!(dacc_work::toil_within_local_radius(&WORK));
//! ```
//!
//! Контроль — тот же заготовок, радиус Local:
//!
//! ```
//! use dacc_core::{BlastRadius, NonEmptyStr, RfcRef, SliceRef, ThrustRef};
//! use dacc_work::{Slice, WorkItem, WorkOrigin};
//! dacc_core::declare_taxonomy! { Subsystem => [Core] }
//! static SLICE: Slice = Slice {
//!     id: "s-001", title: NonEmptyStr::new("срез"), thrust: ThrustRef::__from_scan("t-001"),
//!     outcome: NonEmptyStr::new("исход"), specification: RfcRef::__from_scan("rfc-2026-001"),
//!     max_radius: BlastRadius::Crate,
//! };
//! static WORK: WorkItem = WorkItem {
//!     id: "w-001", title: NonEmptyStr::new("работа"), slice: SliceRef::__from_scan("s-001"),
//!     origin: WorkOrigin::Toil { justification: NonEmptyStr::new("рутина") },
//!     taxon: Subsystem::Core, radius: BlastRadius::Local,
//!     outcome: NonEmptyStr::new("готово"),
//! };
//! const _: () = assert!(dacc_work::toil_within_local_radius(&WORK));
//! ```
