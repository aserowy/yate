use std::{
    io::{stdout, Write},
    path::PathBuf,
};

use tokio::fs;
use yeet_buffer::message::BufferMessage;
use yeet_keymap::message::{KeymapMessage, QuitMode};

use crate::{
    error::AppError,
    event::{Emitter, Message},
    init::{history, mark, qfix},
    model::{DirectoryBufferState, Model, WindowType},
    open,
    task::Task,
    terminal::TerminalWrapper,
    update::{self, viewport},
};

#[derive(Debug)]
pub enum Action {
    EmitMessages(Vec<Message>),
    Load(WindowType, PathBuf, Option<String>),
    ModeChanged,
    Open(PathBuf),
    Quit(QuitMode, Option<String>),
    Resize(u16, u16),
    Task(Task),
    UnwatchPath(PathBuf),
    WatchPath(PathBuf),
}

pub fn emit_keymap(message: KeymapMessage) -> Action {
    Action::EmitMessages(vec![Message::Keymap(message)])
}

#[derive(PartialEq)]
pub enum ActionResult {
    Normal,
    SkipRender,
    Quit(QuitMode),
}

pub struct ExecResult {
    pub result: ActionResult,
    pub remaining_actions: Vec<Action>,
}

#[tracing::instrument(skip(model, emitter, terminal, actions))]
pub async fn preview(
    model: &mut Model,
    emitter: &mut Emitter,
    terminal: &mut TerminalWrapper,
    actions: Vec<Action>,
) -> Result<ExecResult, AppError> {
    execute(true, model, emitter, terminal, actions).await
}

#[tracing::instrument(skip(model, emitter, terminal, actions))]
pub async fn postview(
    model: &mut Model,
    emitter: &mut Emitter,
    terminal: &mut TerminalWrapper,
    actions: Vec<Action>,
) -> Result<ExecResult, AppError> {
    execute(false, model, emitter, terminal, actions).await
}

fn is_preview_action(action: &Action) -> bool {
    match action {
        Action::Load(_, _, _) | Action::Open(_) | Action::Resize(_, _) | Action::Task(_) => true,

        Action::EmitMessages(_)
        | Action::ModeChanged
        | Action::Quit(_, _)
        | Action::UnwatchPath(_)
        | Action::WatchPath(_) => false,
    }
}

async fn execute(
    is_preview: bool,
    model: &mut Model,
    emitter: &mut Emitter,
    terminal: &mut TerminalWrapper,
    actions: Vec<Action>,
) -> Result<ExecResult, AppError> {
    let quit_mode = if is_preview {
        None
    } else {
        contains_quit(&actions)
    };

    let result = if is_preview && contains_emit(&actions) {
        ActionResult::SkipRender
    } else if let Some(mode) = quit_mode {
        ActionResult::Quit(mode)
    } else {
        ActionResult::Normal
    };

    let mut remaining_actions = vec![];
    for action in actions.into_iter() {
        if is_preview != is_preview_action(&action) {
            remaining_actions.push(action);
            continue;
        }

        tracing::debug!("handling action: {:?}", action);

        match action {
            Action::EmitMessages(messages) => {
                emitter.run(Task::EmitMessages(messages));
            }
            Action::Load(window_type, path, selection) => {
                match window_type {
                    WindowType::Current => {
                        model.files.current.state = DirectoryBufferState::Loading;
                        model.files.current.path = path.clone();

                        yeet_buffer::update::update_buffer(
                            &mut model.files.current_vp,
                            &mut model.files.current_cursor,
                            &model.mode,
                            &mut model.files.current.buffer,
                            &BufferMessage::SetContent(Vec::new()),
                        );

                        viewport::set_viewport_dimensions(
                            &mut model.files.current_vp,
                            &model.layout.current,
                        );

                        yeet_buffer::update::update_buffer(
                            &mut model.files.current_vp,
                            &mut model.files.current_cursor,
                            &model.mode,
                            &mut model.files.current.buffer,
                            &BufferMessage::ResetCursor,
                        );

                        emitter.run(Task::EnumerateDirectory(path, selection.clone()));
                    }
                    WindowType::Parent | WindowType::Preview => {
                        update::buffer_type(&window_type, model, path.as_path(), vec![]);

                        if path.is_dir() {
                            emitter.run(Task::EnumerateDirectory(path.clone(), selection.clone()));
                        } else {
                            emitter.run(Task::LoadPreview(path.clone(), model.layout.preview));
                        }
                    }
                };
            }
            Action::ModeChanged => {
                emitter.set_current_mode(model.mode.clone()).await;
            }
            Action::Open(path) => {
                // TODO: check with mime if suspend/resume is necessary?
                emitter.suspend();
                terminal.suspend();

                // TODO: remove flickering (alternate screen leave and cli started)
                open::path(&path).await?;

                emitter.resume();
                terminal.resume()?;
            }
            Action::Quit(mode, stdout_result) => {
                if let Some(stdout_result) = stdout_result {
                    if let Some(target) = &model.settings.selection_to_file_on_open {
                        if let Err(error) = fs::write(target, stdout_result.clone()).await {
                            tracing::error!("Failed to write selection to file: {:?}", error);
                        }
                    }

                    if model.settings.selection_to_stdout_on_open {
                        stdout().lock().write_all(stdout_result.as_bytes())?;
                    }
                }

                match mode {
                    QuitMode::FailOnRunningTasks => {
                        if let Err(error) = history::save_history_to_file(&model.history) {
                            tracing::error!("Failed to save history to file: {:?}", error);
                        }
                        history::optimize_history_file()?;
                        if let Err(error) = mark::save_marks_to_file(&model.marks) {
                            tracing::error!("Failed to save marks to file: {:?}", error);
                        }
                        if let Err(error) = qfix::save_qfix_to_files(&model.qfix) {
                            tracing::error!("Failed to save quick fix to file: {:?}", error);
                        }
                    }
                    QuitMode::Force => {}
                };
            }
            Action::Resize(x, y) => {
                terminal.resize(x, y)?;

                if let Some(path) = &model.files.preview.resolve_path() {
                    emitter.run(Task::LoadPreview(path.to_path_buf(), model.layout.preview));
                }
            }
            Action::Task(task) => emitter.run(task),
            Action::UnwatchPath(path) => {
                if path == PathBuf::default() {
                    continue;
                }

                if let Some(cancellation) = model
                    .current_tasks
                    .get(&Task::EnumerateDirectory(path.clone(), None).to_string())
                {
                    cancellation.token.cancel();
                };

                if let Err(error) = emitter.unwatch(path.as_path()) {
                    tracing::debug!("emitting unwatch path failed: {:?}", error);
                }
            }
            Action::WatchPath(path) => {
                if path == PathBuf::default() {
                    continue;
                }

                if let Err(error) = emitter.watch(path.as_path()) {
                    tracing::error!("emitting watch path failed: {:?}", error);
                }
            }
        }
    }

    Ok(ExecResult {
        result,
        remaining_actions,
    })
}

fn contains_emit(actions: &[Action]) -> bool {
    actions.iter().any(|a| matches!(a, Action::EmitMessages(_)))
}

fn contains_quit(actions: &[Action]) -> Option<QuitMode> {
    actions.iter().find_map(|a| {
        if let Action::Quit(mode, _) = a {
            Some(mode.clone())
        } else {
            None
        }
    })
}
