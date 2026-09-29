use dacc_core::NonEmptyStr;

dacc_work::limitation!("l-gate-delegation-once",
    title: NonEmptyStr::new("Условие делегирования gate_command вычисляется один раз"),
    violated: NonEmptyStr::new(
        "Решение 28 объявляет команду проекта одной проверкой с одним исполнителем, но условие «gate_command задана» повторяется в каждом из шести шагов cargo и в счётчике шагов: решение о делегировании размазано по коду и при добавлении шага может разойтись."
    ),
);
