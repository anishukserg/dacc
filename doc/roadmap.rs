//! Дорожная карта — первая проекция реестра (решение 21). Печатает план из
//! скомпилированного реестра, а не из повторного разбора: данные берутся из
//! констант, которые уже проверил компилятор.

use dacc_doc::{ALL_SLICES, ALL_THRUSTS, ALL_WORK, CLOSED_SLICES, COMMIT, WORK_STATES};

fn main() {
    print!(
        "{}",
        dacc_work::roadmap::render_roadmap(
            ALL_THRUSTS,
            ALL_SLICES,
            ALL_WORK,
            WORK_STATES,
            CLOSED_SLICES,
            COMMIT
        )
    );
}
