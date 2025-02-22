use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use futures::{FutureExt, StreamExt};
use notify::{
    event::{ModifyKind, RenameMode},
    RecommendedWatcher, RecursiveMode, Watcher,
};
use ratatui_image::protocol::Protocol;
use tokio::{
    select,
    sync::{
        mpsc::{self, Receiver},
        Mutex,
    },
};
use tokio_util::sync::CancellationToken;
use yeet_buffer::model::Mode;
use yeet_keymap::{
    conversion,
    message::{KeySequence, KeymapMessage},
    MessageResolver,
};

use crate::{
    error::AppError,
    init::junkyard::get_junkyard_path,
    task::{Task, TaskManager},
};

#[derive(Debug)]
pub struct Envelope {
    pub messages: Vec<Message>,
    pub sequence: KeySequence,
    pub source: MessageSource,
}
impl Envelope {
    pub fn clone_keymap_messages(&self) -> Vec<KeymapMessage> {
        self.messages
            .iter()
            .flat_map(|message| {
                if let Message::Keymap(keymap_message) = message {
                    Some(keymap_message.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum MessageSource {
    Filesystem,
    Task,
    User,
}

pub enum Message {
    Keymap(KeymapMessage),
    EnumerationChanged(PathBuf, Vec<(ContentKind, String)>, Option<String>),
    EnumerationFinished(PathBuf, Vec<(ContentKind, String)>, Option<String>),
    Error(String),
    FdResult(Vec<PathBuf>),
    PathRemoved(PathBuf),
    PathsAdded(Vec<PathBuf>),
    PreviewLoaded(Preview),
    Rerender,
    Resize(u16, u16),
    TaskStarted(String, CancellationToken),
    TaskEnded(String),
    ZoxideResult(PathBuf),
}

impl std::fmt::Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::Keymap(msg) => write!(f, "Keymap({:?})", msg),
            Message::EnumerationChanged(path, _, opt) => {
                write!(f, "EnumerationChanged({:?}, _, {:?})", path, opt)
            }
            Message::EnumerationFinished(path, _, opt) => {
                write!(f, "EnumerationFinished({:?}, {:?})", path, opt)
            }
            Message::Error(err) => write!(f, "Error({:?})", err),
            Message::FdResult(paths) => write!(f, "FdResult({:?})", paths),
            Message::PathRemoved(path) => write!(f, "PathRemoved({:?})", path),
            Message::PathsAdded(paths) => write!(f, "PathsAdded({:?})", paths),
            Message::PreviewLoaded(preview) => write!(f, "PreviewLoaded({:?})", preview),
            Message::Rerender => write!(f, "Rerender"),
            Message::Resize(x, y) => write!(f, "Resize({}, {})", x, y),
            Message::TaskStarted(identifier, _) => write!(f, "TaskStarted({})", identifier),
            Message::TaskEnded(identifier) => write!(f, "TaskEnded({})", identifier),
            Message::ZoxideResult(path) => write!(f, "ZoxideResult({:?})", path),
        }
    }
}

pub enum Preview {
    Content(PathBuf, Vec<String>),
    Image(PathBuf, Protocol),
    None(PathBuf),
}

impl std::fmt::Debug for Preview {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Preview::Content(path, _) => write!(f, "Content({:?})", path),
            Preview::Image(path, _) => write!(f, "Image({:?})", path),
            Preview::None(path) => write!(f, "None({:?})", path),
        }
    }
}

impl Eq for Preview {}

impl PartialEq for Preview {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Preview::Content(p1, _), Preview::Content(p2, _)) => p1 == p2,
            (Preview::Image(p1, _), Preview::Image(p2, _)) => p1 == p2,
            (Preview::None(p1), Preview::None(p2)) => p1 == p2,
            _ => false,
        }
    }
}

// TODO: replace with ansi string
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContentKind {
    Directory,
    File,
}

pub struct Emitter {
    cancellation: CancellationToken,
    crossterm_cancellation: CancellationToken,
    pub receiver: Receiver<Envelope>,
    resolver: Arc<Mutex<MessageResolver>>,
    sender: mpsc::Sender<Envelope>,
    tasks: TaskManager,
    watcher: RecommendedWatcher,
}

impl Emitter {
    pub fn start(cancellation: CancellationToken) -> Self {
        let (sender, receiver) = mpsc::channel(1);
        let internal_sender = sender.clone();

        let (watcher_sender, mut notify_receiver) = mpsc::unbounded_channel();
        let watcher = notify::recommended_watcher(move |res| {
            if let Err(error) = watcher_sender.send(res) {
                tracing::error!("sending watched directory changes failed: {:?}", error);
            }
        })
        .expect("Failed to create watcher");

        let resolver = Arc::new(Mutex::new(MessageResolver::default()));

        let (task_sender, mut task_receiver) = mpsc::channel(1);
        let tasks = TaskManager::new(task_sender, resolver.clone(), cancellation.child_token());

        tokio::spawn(async move {
            loop {
                let notify_event = notify_receiver.recv().fuse();
                let task_event = task_receiver.recv().fuse();

                tokio::select! {
                    Some(Ok(event)) = notify_event => {
                        if let Some(messages) = handle_notify_event(event) {
                            let _ = internal_sender.send(Envelope {
                                messages,
                                sequence: KeySequence::None,
                                source: MessageSource::Filesystem,
                            }).await;
                        }
                    }
                    event = task_event => {
                        if let Some(envelope) = event {
                            let _ = internal_sender.send(envelope).await;
                        }
                    },
                }
            }
        });

        let crossterm_cancellation = cancellation.child_token();
        start_crossterm_listener(
            crossterm_cancellation.clone(),
            resolver.clone(),
            sender.clone(),
        );

        Self {
            cancellation,
            crossterm_cancellation,
            sender,
            tasks,
            receiver,
            resolver,
            watcher,
        }
    }

    pub fn suspend(&mut self) {
        self.crossterm_cancellation.cancel();
    }

    pub fn resume(&mut self) {
        self.crossterm_cancellation = self.cancellation.child_token();

        start_crossterm_listener(
            self.crossterm_cancellation.clone(),
            self.resolver.clone(),
            self.sender.clone(),
        );
    }

    pub fn shutdown(&mut self) {
        self.cancellation.cancel();
    }

    pub async fn set_current_mode(&mut self, mode: Mode) {
        let mut resolver = self.resolver.lock().await;
        resolver.mode = mode;
    }

    pub fn unwatch(&mut self, path: &Path) -> Result<(), AppError> {
        if path != get_junkyard_path()? {
            Ok(self.watcher.unwatch(path)?)
        } else {
            Ok(())
        }
    }

    pub fn watch(&mut self, path: &Path) -> Result<(), AppError> {
        Ok(self.watcher.watch(path, RecursiveMode::NonRecursive)?)
    }

    pub fn run(&mut self, task: Task) {
        match self.tasks.sender.send(task) {
            Ok(_) => {}
            Err(err) => tracing::error!("failed to send task: {:?}", err),
        };
    }
}

fn start_crossterm_listener(
    cancellation: CancellationToken,
    resolver_mutex: Arc<Mutex<MessageResolver>>,
    sender: mpsc::Sender<Envelope>,
) {
    tokio::spawn(async move {
        let mut reader = crossterm::event::EventStream::new();

        loop {
            let crossterm_event = reader.next().fuse();

            select! {
                _ = cancellation.cancelled() => break,
                Some(Ok(event)) = crossterm_event => {
                    if let Some(envelope) = handle_crossterm_event(&resolver_mutex, event).await {
                        let _ = sender.send(envelope).await;
                    }
                }
            }
        }
    });
}

async fn handle_crossterm_event(
    resolver_mutex: &Arc<Mutex<MessageResolver>>,
    event: crossterm::event::Event,
) -> Option<Envelope> {
    match event {
        crossterm::event::Event::Key(key) => {
            if let Some(key) = conversion::to_key(&key) {
                let mut resolver = resolver_mutex.lock().await;
                let (messages, sequence) = resolver.add_key(key);
                return Some(Envelope {
                    messages: messages.into_iter().map(Message::Keymap).collect(),
                    sequence,
                    source: MessageSource::User,
                });
            }

            None
        }
        crossterm::event::Event::Resize(x, y) => Some(Envelope {
            messages: vec![Message::Resize(x, y)],
            sequence: KeySequence::None,
            source: MessageSource::User,
        }),
        crossterm::event::Event::FocusLost
        | crossterm::event::Event::FocusGained
        | crossterm::event::Event::Paste(_)
        | crossterm::event::Event::Mouse(_) => None,
    }
}

#[tracing::instrument]
fn handle_notify_event(event: notify::Event) -> Option<Vec<Message>> {
    if event.need_rescan() {
        // TODO: Refresh directory
    }

    match event.kind {
        notify::EventKind::Create(_) => Some(
            event
                .paths
                .iter()
                .map(|p| Message::PathsAdded(vec![p.clone()]))
                .collect(),
        ),
        notify::EventKind::Modify(ModifyKind::Name(rename_mode)) => match rename_mode {
            RenameMode::Both => {
                if event.paths.len() == 2 {
                    Some(vec![
                        Message::PathRemoved(event.paths[0].clone()),
                        Message::PathsAdded(vec![event.paths[1].clone()]),
                    ])
                } else {
                    tracing::warn!("event is invalid: {:?}", event);
                    None
                }
            }
            RenameMode::From => {
                if event.paths.len() == 1 {
                    Some(vec![Message::PathRemoved(event.paths[0].clone())])
                } else {
                    tracing::warn!("event is invalid: {:?}", event);
                    None
                }
            }
            RenameMode::To => {
                if event.paths.len() == 1 {
                    Some(vec![Message::PathsAdded(vec![event.paths[0].clone()])])
                } else {
                    tracing::warn!("event is invalid: {:?}", event);
                    None
                }
            }
            RenameMode::Any => {
                if event.paths.len() == 1 {
                    let path = event.paths[0].clone();
                    if path.exists() {
                        Some(vec![Message::PathsAdded(vec![path])])
                    } else {
                        Some(vec![Message::PathRemoved(path)])
                    }
                } else {
                    tracing::warn!("event is invalid: {:?}", event);
                    None
                }
            }
            RenameMode::Other => {
                tracing::trace!("missed handle for notify event: {:?}", event);
                None
            }
        },
        notify::EventKind::Remove(_) => Some(
            event
                .paths
                .iter()
                .map(|p| Message::PathRemoved(p.clone()))
                .collect(),
        ),
        notify::EventKind::Any
        | notify::EventKind::Access(_)
        | notify::EventKind::Modify(_)
        | notify::EventKind::Other => {
            tracing::trace!("missed handle for notify event: {:?}", event);
            None
        }
    }
}
