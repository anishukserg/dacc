use dacc_core::NonEmptyStr;
use dacc_knowledge::{Enforced, InvariantStatus};

dacc_knowledge::invariant!("i-access-answers",
    specification: crate::rfc::rfc_2026_003,
    status: InvariantStatus::Enforced(Enforced::Adversarial {
        bypass: NonEmptyStr::new(
            "вывод слоя доступа мог пропустить стадию свёртки в счётчике, и ответ оставался бы правдоподобным"
        ),
        tests: &[crate::anchor::access_answers_enforced],
    }),
    statement: NonEmptyStr::new(
        "Состояние работы в ответах слоя доступа — свёртка журнала, а не поле записи: счётчик карты ведёт каждую стадию свёртки, и ответ не утверждает стадию вне её."
    ),
    rationale: "У записи работы нет поля состояния, а WorkState порождается свёрткой событий при сборке реестра: у ответа нет другого источника стадии, кроме журнала. Обход опробован тестом access-answers-enforced: сумма счётчиков стадий равна числу работ карты, и пропущенная стадия краснеет.",
    enforced_by: &[crate::anchor::work_state_from_fold],
    tests: &[crate::anchor::access_answers_enforced],
);
