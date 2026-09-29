//! Календарная проверка дат без зависимостей.
//!
//! Единственный источник истины для вопроса «существует ли такая дата»: его
//! используют и макрос `date!` (компиляция реестра), и разбор времени журнала
//! (`dacc_journal::time::is_timestamp`). Правило одно — расхождение невозможно.
//!
//! Григорианский календарь в пролептическом виде, как у `chrono::NaiveDate`:
//! месяц 1..=12, день 1..=число дней месяца, високосный год по правилу
//! «делится на 4, но не на 100, кроме делящихся на 400».

/// Високосный ли год по григорианскому правилу.
pub const fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Число дней в месяце; 0 для месяца вне 1..=12.
pub const fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

/// Существует ли такая дата в пролептическом григорианском календаре.
pub const fn is_valid_ymd(year: i32, month: u32, day: u32) -> bool {
    month >= 1 && month <= 12 && day >= 1 && day <= days_in_month(year, month)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leap_years_follow_the_gregorian_rule() {
        for leap in [2000, 2004, 2024, 2400] {
            assert!(is_leap_year(leap), "{leap}");
        }
        for common in [1900, 2100, 2023, 2026] {
            assert!(!is_leap_year(common), "{common}");
        }
    }

    #[test]
    fn month_lengths_respect_february() {
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2026, 2), 28);
        assert_eq!(days_in_month(2026, 4), 30);
        assert_eq!(days_in_month(2026, 1), 31);
        assert_eq!(days_in_month(2026, 13), 0);
    }

    #[test]
    fn impossible_dates_are_rejected() {
        for bad in [
            (2026, 2, 29),
            (2026, 4, 31),
            (2026, 13, 1),
            (2026, 0, 1),
            (2026, 1, 0),
            (2026, 1, 32),
        ] {
            assert!(!is_valid_ymd(bad.0, bad.1, bad.2), "{bad:?}");
        }
        for ok in [(2024, 2, 29), (2026, 2, 28), (2026, 12, 31), (2026, 1, 1)] {
            assert!(is_valid_ymd(ok.0, ok.1, ok.2), "{ok:?}");
        }
    }
}
