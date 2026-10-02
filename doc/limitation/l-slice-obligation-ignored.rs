use dacc_core::NonEmptyStr;

dacc_work::limitation!("l-slice-obligation-ignored",
    title: NonEmptyStr::new("slice close закрывал срез с непогашенным обязательством"),
    violated: NonEmptyStr::new(
        "slice close отказывает, пока обязательство, на которое ссылаются работы среза, не погашено приземлённой работой с происхождением WorkOrigin::Obligation (решение 35)"
    ),
);
