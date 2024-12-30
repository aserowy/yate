use crate::{action::Action, model::Tasks};

pub fn delete(tasks: &mut Tasks, id: u16) -> Vec<Action> {
    if let Some((_, task)) = tasks.running.iter().find(|(_, task)| task.id == id) {
        task.token.cancel();
    }
    Vec::new()
}
