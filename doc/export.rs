//! Экспорт реестра (решение 21): JSON всего графа, XML среза по вопросу или
//! Markdown-сайт для mdBook, порождённые из скомпилированного реестра, а не из
//! повторного разбора.

use dacc_doc::taxonomy::channel;
use dacc_doc::{
    ALL, ALL_SLICES, ALL_SPECS, ALL_THRUSTS, ALL_WORK, CLOSED_SLICES, COMMIT, WORK_STATES,
};
use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("json") => {
            let (decisions, specs) = published();
            print!(
                "{}",
                dacc_access::export_graph(
                    &decisions,
                    &specs,
                    ALL_THRUSTS,
                    ALL_SLICES,
                    ALL_WORK,
                    WORK_STATES,
                    CLOSED_SLICES,
                    COMMIT,
                )
            );
        }
        Some("xml") => {
            let (decisions, specs) = published();
            let question = args[1..].join(" ");
            print!(
                "{}",
                dacc_access::export_slice(&decisions, &specs, &question, COMMIT)
            );
        }
        Some("markdown") => {
            let (decisions, specs) = published();
            let dir = args.get(1).map_or("site/src", String::as_str);
            let site = dacc_access::render_site(&decisions, &specs);
            if let Err(error) = write_site(dir, &site) {
                eprintln!("markdown: {error}");
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!("usage: export json | export xml <question> | export markdown [<dir>]");
            std::process::exit(2);
        }
    }
}

/// Решения и спецификации публичного канала; ошибка — ссылка через границу
/// канала (решение 21).
fn published() -> dacc_access::ChannelDocuments<'static> {
    match dacc_access::publish_channel(ALL, ALL_SPECS, channel::Public) {
        Ok(pair) => pair,
        Err(error) => {
            eprintln!("publish: {error}");
            std::process::exit(1);
        }
    }
}

/// Записывает оглавление и страницы сайта в `dir`.
fn write_site(dir: &str, site: &dacc_access::Site) -> std::io::Result<()> {
    fs::create_dir_all(dir)?;
    fs::write(format!("{dir}/SUMMARY.md"), &site.summary)?;
    for (path, content) in &site.pages {
        let full = format!("{dir}/{path}");
        if let Some(parent) = Path::new(&full).parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(full, content)?;
    }
    Ok(())
}
