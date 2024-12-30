use std::cmp::Ordering;

use tokio_util::sync::CancellationToken;

use crate::{
    action::Action,
    model::{CurrentTask, Tasks},
};

pub fn add(tasks: &mut Tasks, identifier: String, cancellation: CancellationToken) -> Vec<Action> {
    let id = next_id(tasks);

    if let Some(replaced_task) = tasks.running.insert(
        identifier.clone(),
        CurrentTask {
            token: cancellation,
            id,
            external_id: identifier,
        },
    ) {
        replaced_task.token.cancel();
    }
    Vec::new()
}

fn next_id(tasks: &mut Tasks) -> u16 {
    let mut next_id = if tasks.latest_id >= 9999 {
        1
    } else {
        tasks.latest_id + 1
    };

    let mut running_ids: Vec<u16> = tasks.running.values().map(|task| task.id).collect();
    running_ids.sort();

    for id in running_ids {
        match next_id.cmp(&id) {
            Ordering::Equal => next_id += 1,
            Ordering::Greater => break,
            Ordering::Less => {}
        }
    }

    tasks.latest_id = next_id;

    next_id
}

pub fn remove(tasks: &mut Tasks, identifier: String) -> Vec<Action> {
    if let Some(task) = tasks.running.remove(&identifier) {
        task.token.cancel();
    }
    Vec::new()
}
