use yeet_buffer::{
    message::BufferMessage,
    model::{
        undo::{consolidate_modifications, BufferChanged},
        BufferResult, Mode,
    },
};

use crate::{
    action::Action,
    model::{junkyard::JunkYard, App, Buffer},
    task::Task,
};

use super::{app, junkyard::trash_to_junkyard, selection::get_current_selected_bufferline};

#[tracing::instrument(skip(app))]
pub fn changes(app: &mut App, junk: &mut JunkYard, mode: &Mode) -> Vec<Action> {
    let (vp, cursor, buffer) = match app::get_focused_mut(app) {
        (vp, cursor, Buffer::FileTree(it)) => (vp, cursor, it),
        (_vp, _cursor, Buffer::_Text(_)) => todo!(),
    };

    let selection = get_current_selected_bufferline(buffer).map(|line| line.content.clone());

    let mut content: Vec<_> = buffer.current.buffer.lines.drain(..).collect();
    content.retain(|line| !line.content.is_empty());

    yeet_buffer::update(
        Some(vp),
        Some(cursor),
        mode,
        &mut buffer.current.buffer,
        vec![&BufferMessage::SetContent(content)],
    );

    if let Some(selection) = selection {
        yeet_buffer::update(
            vp,
            Some(cursor),
            mode,
            &mut buffer.current.buffer,
            vec![&BufferMessage::SetCursorToLineContent(
                selection.to_stripped_string(),
            )],
        );
    }

    let result = yeet_buffer::update(
        vp,
        Some(cursor),
        mode,
        &mut buffer.current.buffer,
        vec![&BufferMessage::SaveBuffer],
    );

    let mut actions = Vec::new();
    for br in result {
        if let BufferResult::Changes(modifications) = br {
            let path = &buffer.current.path;
            let mut trashes = Vec::new();
            for modification in consolidate_modifications(&modifications) {
                match modification {
                    BufferChanged::LineAdded(_, name) => {
                        if !name.is_empty() {
                            actions.push(Action::Task(Task::AddPath(
                                path.join(name.to_stripped_string()),
                            )))
                        }
                    }
                    BufferChanged::LineRemoved(_, name) => {
                        trashes.push(path.join(name.to_stripped_string()));
                    }
                    BufferChanged::Content(_, old_name, new_name) => {
                        let task = if new_name.is_empty() {
                            Task::DeletePath(path.join(old_name.to_stripped_string()))
                        } else {
                            Task::RenamePath(
                                path.join(old_name.to_stripped_string()),
                                path.join(new_name.to_stripped_string()),
                            )
                        };
                        actions.push(Action::Task(task));
                    }
                }
            }

            if !trashes.is_empty() {
                let (transaction, obsolete) = trash_to_junkyard(junk, trashes);
                for entry in transaction.entries {
                    actions.push(Action::Task(Task::TrashPath(entry)));
                }

                if let Some(obsolete) = obsolete {
                    for entry in obsolete.entries {
                        actions.push(Action::Task(Task::DeleteJunkYardEntry(entry)));
                    }
                }
            }
        }
    }
    actions
}
