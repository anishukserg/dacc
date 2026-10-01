use dacc_core::NonEmptyStr;

dacc_work::limitation!("l-red-before-claim",
    title: NonEmptyStr::new("Падение до починки остаётся заявлением"),
    violated: NonEmptyStr::new(
        "RedBefore подтверждает, что названный тест существует в дереве и исполняется калиткой, но само его падение до починки механически не проверяется: состояние «до» не сохраняется."
    ),
);
