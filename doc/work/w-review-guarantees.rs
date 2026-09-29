use crate::taxonomy::Subsystem;
use dacc_core::{taxon, BlastRadius, NonEmptyStr};
use dacc_work::WorkOrigin;

dacc_work::work!("w-review-guarantees",
    title: NonEmptyStr::new("Ревью: честные гарантии дат и обязательств, дефекты инструмента"),
    slice: crate::slice::s_review_guarantees,
    origin: WorkOrigin::Divergence {
        specification: crate::rfc::rfc_2026_002,
        violated: NonEmptyStr::new(
            "Объявлено «несуществующая дата — ошибка компиляции» и «молчаливое забывание обязательства невыразимо», но журнал принимал 31 февраля, а погашение обязательств не было подключено к сборке; work new писал замер мимо настроенного корня и оставлял файл при отказе."
        ),
    },
    taxon: taxon!(Subsystem, Methodology),
    radius: BlastRadius::Crate,
    outcome: NonEmptyStr::new(
        "Единый календарный валидатор date в dacc-core используют и макрос date!, и разбор времени журнала; обязательства проверяются константным утверждением obligations_redeemed; work new пишет замер трения в <doc>/work-new.tsv и откатывает файл работы при отказе замера; close_slice использует параметр slice; JSON-вывод экранирует DEL."
    ),
);
