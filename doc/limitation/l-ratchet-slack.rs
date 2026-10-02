use dacc_core::NonEmptyStr;

dacc_work::limitation!("l-ratchet-slack",
    title: NonEmptyStr::new("Заморозка baseline держала люфт"),
    violated: NonEmptyStr::new(
        "Baseline держит точный размер долга: уменьшение фиксируется явно, и файл не может незаметно вернуться к замороженному размеру (решение 40)."
    ),
);
