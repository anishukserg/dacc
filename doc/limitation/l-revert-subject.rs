use dacc_core::NonEmptyStr;

dacc_work::limitation!("l-revert-subject",
    title: NonEmptyStr::new("msg-check отвергал revert за форму темы"),
    violated: NonEmptyStr::new(
        "Revert \"[ТИП](область): суть\" проверяется как тема, которую он отменяет: форма, тип, область и точка берутся из внутренней темы, а обёртка git не мешает откату"
    ),
);
