use dacc_core::NonEmptyStr;

dacc_work::limitation!("l-toil-radius",
    title: NonEmptyStr::new("Toil не выходит за пределы Local"),
    violated: NonEmptyStr::new(
        "происхождение Toil объявляет «рутину», но радиус работы никак не ограничен: оправдание непустой строкой допускает тронуть публичный API без среза и решения"
    ),
);
