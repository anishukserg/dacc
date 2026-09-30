use dacc_core::NonEmptyStr;

dacc_work::upgrade!("u-0-4-0-work-start",
    from: NonEmptyStr::new("0.3.0"),
    to: NonEmptyStr::new("0.4.0"),
    subject: NonEmptyStr::new("work start no longer writes a commit"),
    how: NonEmptyStr::new(
        "the ceremony is commit + work land; work start only validates"
    ),
);
