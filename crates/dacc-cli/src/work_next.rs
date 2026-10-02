//! Задачная модель агентов (работа w-work-next): `cargo dacc work next`
//! берёт следующую задачу с допуском и держит WIP-лимит.
//!
//! Допуск — запланированная работа открытого среза в порядке плана; взятая
//! задача получает событие `started` под межпроцессным локом журнала, и
//! WIP-лимит из настройки `wip_limit` отказывает переполнению кодом причины,
//! называя занятые работы.

use crate::code;
use crate::config;
use crate::format::Format;
use crate::work::{
    message, record_and_commit_under_lock, refused, stage_text, usage, Context, Refusal,
};
use dacc_journal::{time, Event, Kind, Stage, Subject};
use std::cell::Cell;

/// `cargo dacc work next [--format json]` — допуск следующей задачи.
pub(crate) fn next(format: Format) -> Result<u8, Refusal> {
    // xml поддержан map и where — как в слое доступа (работа w-machine-refusals).
    if format == Format::Xml {
        return Err(usage(
            code::FORMAT_CHOICE,
            "xml is supported by map and where",
        ));
    }
    // Ранний отказ вне лока: журнал, который не сворачивается, не
    // допускает задач.
    let context = Context::open()?;
    let taken: Cell<Option<(String, String, usize, usize)>> = Cell::new(None);
    let code = record_and_commit_under_lock(&context.repo, || {
        let context = Context::open()?;
        let works = context.works()?;
        let limit = context.repo.config.wip_limit;
        let inflight: Vec<String> = works
            .iter()
            .filter(|(number, _)| context.journal.stage(number) == Stage::Started)
            .map(|(number, _)| number.clone())
            .collect();
        if inflight.len() >= limit {
            return Err(refused(
                code::WIP_LIMIT,
                format!(
                    "the wip limit {limit} is reached: {} is in flight; land or drop a work first",
                    inflight.join(", ")
                ),
            ));
        }
        // Допуск пропускает дефектную запись без таксона и берёт следующую:
        // одна испорченная запись не ставит всю очередь (работа
        // w-machine-refusals).
        let candidate = works.iter().find(|(number, work)| {
            context.journal.stage(number) == Stage::Planned
                && work.area.is_some()
                && work
                    .slice
                    .as_deref()
                    .is_none_or(|slice| !context.journal.closed_slices.contains_key(slice))
        });
        let Some((number, work)) = candidate else {
            return Err(refused(
                code::NOTHING_TO_TAKE,
                "no admissible work: every planned work is finished, belongs to a closed slice, or its record has no taxon subsystem",
            ));
        };
        let id = number.clone();
        let area = work.area(&id)?.to_owned();
        let event = Event::new(Subject::Work(number.clone()), time::now(), Kind::Started);
        let text = message(
            &config::fill(&context.repo.config.subject_started, &area, &id),
            &work.title,
            &format!("Dacc-Work: {id}"),
            &[],
        );
        taken.set(Some((id, work.title.clone(), inflight.len() + 1, limit)));
        Ok((vec![event], text))
    })?;
    if code != 0 {
        return Ok(code);
    }
    let Some((id, title, wip, limit)) = taken.into_inner() else {
        return Ok(0);
    };
    match format {
        Format::Json => println!(
            "{{\"schema\": \"dacc-next\", \"work\": {{\"id\": {}, \"state\": {}, \"title\": {}}}, \"wip\": {{\"taken\": {wip}, \"limit\": {limit}}}}}",
            crate::format::string(&id),
            crate::format::string(stage_text(Stage::Started)),
            crate::format::string(&title)
        ),
        Format::Text | Format::Xml => {
            println!("work {id} taken: {title}");
            println!("wip: {wip} of {limit}");
        }
    }
    Ok(0)
}
