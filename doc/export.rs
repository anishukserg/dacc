//! Экспорт реестра (решение 21): JSON всего графа или XML среза по вопросу,
//! порождённые из скомпилированного реестра, а не из повторного разбора.

use slipway_doc::{
    ALL, ALL_SLICES, ALL_SPECS, ALL_THRUSTS, ALL_WORK, CLOSED_SLICES, COMMIT, WORK_STATES,
};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("json") => print!(
            "{}",
            slipway_access::export_graph(
                ALL,
                ALL_SPECS,
                ALL_THRUSTS,
                ALL_SLICES,
                ALL_WORK,
                WORK_STATES,
                CLOSED_SLICES,
                COMMIT,
            )
        ),
        Some("xml") => {
            let question = args[1..].join(" ");
            print!(
                "{}",
                slipway_access::export_slice(ALL, ALL_SPECS, &question, COMMIT)
            );
        }
        _ => {
            eprintln!("usage: export json | export xml <question>");
            std::process::exit(2);
        }
    }
}
