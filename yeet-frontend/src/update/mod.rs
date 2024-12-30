use std::{cmp::Ordering, path::Path};

use yeet_buffer::{
    message::BufferMessage,
    model::{ansi::Ansi, viewport::ViewPort, BufferLine, Mode, TextBuffer},
};
use yeet_keymap::message::{KeySequence, KeymapMessage, PrintContent};

use crate::{
    action::Action,
    event::{Envelope, Message, Preview},
    model::{
        history::History, App, Buffer, FileTreeBuffer, FileTreeBufferSection,
        FileTreeBufferSectionBuffer, Model, State,
    },
    settings::Settings,
    terminal::TerminalWrapper,
};

pub mod app;
mod command;
pub mod commandline;
mod cursor;
mod enumeration;
pub mod history;
pub mod junkyard;
mod mark;
mod mode;
mod modify;
mod navigate;
mod open;
mod path;
mod qfix;
mod register;
mod save;
mod search;
mod selection;
mod settings;
mod sign;
mod task;
mod viewport;
pub mod window;

const SORT: fn(&BufferLine, &BufferLine) -> Ordering = |a, b| {
    a.content
        .to_stripped_string()
        .to_ascii_uppercase()
        .cmp(&b.content.to_stripped_string().to_ascii_uppercase())
};

#[tracing::instrument(skip(model, terminal))]
pub fn model(terminal: &TerminalWrapper, model: &mut Model, envelope: Envelope) -> Vec<Action> {
    match &envelope.sequence {
        KeySequence::Completed(_) => model.app.commandline.key_sequence.clear(),
        KeySequence::Changed(sequence) => {
            sequence.clone_into(&mut model.app.commandline.key_sequence)
        }
        KeySequence::None => {}
    };

    settings::update(model);
    commandline::update(&mut model.app.commandline, &model.state.modes.current, None);

    let keymaps: Vec<_> = envelope.clone_keymap_messages();
    register::start_scope(
        &model.state.modes.current,
        &mut model.state.register,
        &keymaps,
    );

    let sequence = envelope.sequence.clone();
    let actions = envelope
        .messages
        .into_iter()
        .flat_map(|message| {
            update_with_message(&mut model.app, &mut model.state, &model.settings, message)
        })
        .collect();

    let size = terminal.size().expect("Failed to get terminal size");
    match window::update(&mut model.app, size) {
        Ok(_) => {}
        Err(err) => tracing::error!("window update failed with error: {}", err),
    };

    register::finish_scope(
        &model.state.modes.current,
        &mut model.state.register,
        &sequence,
        &keymaps,
    );

    actions
}

#[tracing::instrument(skip_all)]
fn update_with_message(
    app: &mut App,
    state: &mut State,
    settings: &Settings,
    message: Message,
) -> Vec<Action> {
    match message {
        Message::EnumerationChanged(path, contents, selection) => enumeration::change(
            state,
            app.buffers.values_mut().collect(),
            &path,
            &contents,
            &selection,
        ),
        Message::EnumerationFinished(path, contents, selection) => enumeration::finish(
            state,
            app.buffers.values_mut().collect(),
            &path,
            &contents,
            &selection,
        ),
        Message::Error(error) => commandline::print(
            &mut app.commandline,
            &mut state.modes,
            &[PrintContent::Error(error.to_string())],
        ),
        Message::FdResult(paths) => {
            qfix::add(&mut state.qfix, app.buffers.values_mut().collect(), paths)
        }
        Message::Keymap(msg) => update_with_keymap_message(app, state, settings, &msg),
        Message::PathRemoved(path) => path::remove(
            &state.history,
            &mut state.junk,
            &state.modes.current,
            app.buffers.values_mut().collect(),
            &path,
        ),
        Message::PathsAdded(paths) => path::add(
            &state.history,
            &state.marks,
            &state.qfix,
            &state.modes.current,
            app.buffers.values_mut().collect(),
            &paths,
        )
        .into_iter()
        .chain(junkyard::add(&mut state.junk, &paths).into_iter())
        .collect(),
        Message::PreviewLoaded(content) => update_preview(
            &state.history,
            &state.modes.current,
            app.buffers.values_mut().collect(),
            content,
        ),
        Message::Rerender => Vec::new(),
        Message::Resize(x, y) => vec![Action::Resize(x, y)],
        Message::TaskStarted(id, cancellation) => task::add(&mut state.tasks, id, cancellation),
        Message::TaskEnded(id) => task::remove(&mut state.tasks, id),
        Message::ZoxideResult(path) => {
            let id = app::get_next_buffer_id(app);
            if app
                .buffers
                .insert(id, Buffer::FileTree(Box::new(FileTreeBuffer::default())))
                .is_some()
            {
                tracing::error!("Buffer with id {} already exists", id);
            };

            app::set_focused_buffer(app, id);
            navigate::path(app, &state.history, path.as_ref())
        }
    }
}

#[tracing::instrument(skip_all)]
pub fn update_with_keymap_message(
    app: &mut App,
    state: &mut State,
    settings: &Settings,
    msg: &KeymapMessage,
) -> Vec<Action> {
    match msg {
        KeymapMessage::Buffer(msg) => update_with_buffer_message(app, state, msg),
        KeymapMessage::ClearSearchHighlight => search::clear(app.buffers.values_mut().collect()),
        KeymapMessage::DeleteMarks(mrks) => {
            mark::delete(&mut state.marks, app.buffers.values_mut().collect(), mrks)
        }
        KeymapMessage::ExecuteCommand => {
            commandline::update_on_execute(app, &mut state.register, &mut state.modes)
        }
        KeymapMessage::ExecuteCommandString(command) => command::execute(app, state, command),
        KeymapMessage::ExecuteKeySequence(key_sequence) => {
            state.remaining_keysequence.replace(key_sequence.clone());
            Vec::new()
        }
        KeymapMessage::ExecuteRegister(rgstr) => register::replay(&mut state.register, rgstr),
        KeymapMessage::LeaveCommandMode => {
            commandline::leave(app, &mut state.register, &state.modes)
        }
        KeymapMessage::NavigateToMark(char) => {
            navigate::mark(app, &state.history, &state.marks, char)
        }
        KeymapMessage::NavigateToParent => navigate::parent(app),
        KeymapMessage::NavigateToPath(path) => navigate::path(app, &state.history, path),
        KeymapMessage::NavigateToPathAsPreview(path) => {
            navigate::path_as_preview(app, &state.history, path)
        }
        KeymapMessage::NavigateToSelected => navigate::selected(app, &mut state.history),
        KeymapMessage::OpenSelected => match app::get_focused_mut(app) {
            (_, _, Buffer::FileTree(it)) => open::selected(settings, &state.modes.current, it),
            (_, _, Buffer::_Text(_)) => todo!(),
        },
        KeymapMessage::PasteFromJunkYard(entry_id) => match app::get_focused_mut(app) {
            (_, _, Buffer::FileTree(it)) => junkyard::paste(&state.junk, it, entry_id),
            (_, _, Buffer::_Text(_)) => todo!(),
        },
        KeymapMessage::Print(content) => {
            commandline::print(&mut app.commandline, &mut state.modes, content)
        }
        KeymapMessage::ReplayMacro(char) => register::replay_macro(&mut state.register, char),
        KeymapMessage::SetMark(char) => mark::add(app, &mut state.marks, *char),
        KeymapMessage::StartMacro(identifier) => {
            mode::print_recording(&mut app.commandline, &mut state.modes, *identifier)
        }
        KeymapMessage::StopMacro => mode::print_mode(&mut app.commandline, &mut state.modes),
        KeymapMessage::ToggleQuickFix => qfix::toggle(app, &mut state.qfix),
        KeymapMessage::Quit(mode) => vec![Action::Quit(mode.clone(), None)],
        KeymapMessage::YankPathToClipboard => match app::get_focused_mut(app) {
            (_, _, Buffer::FileTree(it)) => selection::copy_to_clipboard(&mut state.register, it),
            (_, _, Buffer::_Text(_)) => todo!(),
        },
        KeymapMessage::YankToJunkYard(repeat) => match app::get_focused_mut(app) {
            (_, _, Buffer::FileTree(it)) => junkyard::yank(&mut state.junk, it, repeat),
            (_, _, Buffer::_Text(_)) => todo!(),
        },
    }
}

#[tracing::instrument(skip_all)]
pub fn update_with_buffer_message(
    app: &mut App,
    state: &mut State,
    msg: &BufferMessage,
) -> Vec<Action> {
    match msg {
        BufferMessage::ChangeMode(from, to) => mode::change(app, state, from, to),
        BufferMessage::Modification(repeat, modification) => match &mut state.modes.current {
            Mode::Command(_) => commandline::modify(app, &mut state.modes, repeat, modification),
            Mode::Insert | Mode::Normal => {
                modify::buffer(app, &state.modes.current, repeat, modification)
            }
            Mode::Navigation => Vec::new(),
        },
        BufferMessage::MoveCursor(rpt, mtn) => match &mut state.modes.current {
            Mode::Command(_) => {
                commandline::update(&mut app.commandline, &state.modes.current, Some(msg))
            }
            Mode::Insert | Mode::Navigation | Mode::Normal => {
                cursor::relocate(app, state, rpt, mtn)
            }
        },
        BufferMessage::MoveViewPort(mtn) => match &state.modes.current {
            Mode::Command(_) => {
                commandline::update(&mut app.commandline, &state.modes.current, Some(msg))
            }
            Mode::Insert | Mode::Navigation | Mode::Normal => {
                viewport::relocate(app, &state.history, &state.modes.current, mtn)
            }
        },
        BufferMessage::SaveBuffer => save::changes(app, &mut state.junk, &state.modes.current),

        BufferMessage::RemoveLine(_)
        | BufferMessage::ResetCursor
        | BufferMessage::SetContent(_)
        | BufferMessage::SetCursorToLineContent(_)
        | BufferMessage::SortContent(_)
        | BufferMessage::UpdateViewPortByCursor => unreachable!(),
    }
}

pub fn update_preview(
    history: &History,
    mode: &Mode,
    buffers: Vec<&mut Buffer>,
    content: Preview,
) -> Vec<Action> {
    match content {
        Preview::Content(path, content) => {
            tracing::trace!("updating preview buffer: {:?}", path);

            let content: Vec<_> = content
                .iter()
                .map(|s| BufferLine {
                    content: Ansi::new(s),
                    ..Default::default()
                })
                .collect();

            for buffer in buffers {
                let buffer = match buffer {
                    Buffer::FileTree(it) => it,
                    Buffer::_Text(_) => continue,
                };

                buffer_type(
                    history,
                    mode,
                    buffer,
                    &FileTreeBufferSection::Preview,
                    &path,
                    content.clone(),
                );
            }
        }
        Preview::Image(_path, _protocol) => {
            for buffer in buffers {
                let _buffer = match buffer {
                    Buffer::FileTree(it) => it,
                    Buffer::_Text(_) => continue,
                };

                // TODO: protocol into arc
                // buffer.preview = FileTreeBufferSectionBuffer::Image(path.clone(), protocol);
            }
        }
        Preview::None(_path) => {
            for buffer in buffers {
                let buffer = match buffer {
                    Buffer::FileTree(it) => it,
                    Buffer::_Text(_) => continue,
                };
                buffer.preview = FileTreeBufferSectionBuffer::None;
            }
        }
    }
    Vec::new()
}

pub fn buffer_type(
    history: &History,
    mode: &Mode,
    buffer: &mut FileTreeBuffer,
    section: &FileTreeBufferSection,
    path: &Path,
    content: Vec<BufferLine>,
) {
    let mut text_buffer = TextBuffer::default();

    let cursor = match section {
        FileTreeBufferSection::Parent => buffer.parent_cursor,
        FileTreeBufferSection::Preview => buffer.preview_cursor,
        FileTreeBufferSection::Current => unreachable!(),
    };

    yeet_buffer::update(
        &mut None,
        &mut cursor,
        mode,
        &mut text_buffer,
        vec![
            &BufferMessage::SetContent(content.to_vec()),
            &BufferMessage::ResetCursor,
        ],
    );

    if let Some(cursor) = cursor {
        cursor.hide_cursor_line = true;
    }

    if path.is_dir() {
        cursor::set_cursor_index_with_history(
            history,
            &mut viewport,
            &mut cursor,
            mode,
            &mut text_buffer,
            path,
        );
    }

    let buffer_type = FileTreeBufferSectionBuffer::Text(path.to_path_buf(), text_buffer);
    match section {
        FileTreeBufferSection::Parent => buffer.parent = buffer_type,
        FileTreeBufferSection::Preview => buffer.preview = buffer_type,
        FileTreeBufferSection::Current => unreachable!(),
    };
}
