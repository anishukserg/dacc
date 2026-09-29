use dacc_core::NonEmptyStr;

dacc_work::limitation!("l-observable-violations",
    title: NonEmptyStr::new("Нарушения скана не называют способ исправления"),
    violated: NonEmptyStr::new(
        "Скан ловит переименования и удаления, но его сообщения называют не все сущности и не называют способ исправления: часть нарушений без строки, а подсказка «как чинить» отсутствует вовсе."
    ),
);
