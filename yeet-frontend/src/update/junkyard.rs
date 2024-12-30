use std::{
    cmp::Reverse,
    path::{Path, PathBuf},
    time,
};

use crate::{
    action::Action,
    model::{
        junkyard::{FileEntry, FileEntryStatus, FileEntryType, FileTransaction, JunkYard},
        FileTreeBuffer,
    },
    task::Task,
};

pub fn add(junk: &mut JunkYard, paths: &Vec<PathBuf>) -> Vec<Action> {
    let mut actions = Vec::new();
    for path in paths {
        if path.starts_with(&junk.path) {
            if let Some(obsolete) = add_or_update_junkyard_entry(junk, path) {
                for entry in obsolete.entries {
                    actions.push(Action::Task(Task::DeleteJunkYardEntry(entry)));
                }
            }
        }
    }

    actions
}

pub fn add_or_update_junkyard_entry(
    junkyard: &mut JunkYard,
    path: &Path,
) -> Option<FileTransaction> {
    if let Some((id, file, target)) = decompose_compression_path(path) {
        if junkyard.yanked.as_ref().is_some_and(|entry| entry.id == id) {
            if let Some(transaction) = junkyard.yanked.as_mut() {
                let entry = transaction
                    .entries
                    .iter_mut()
                    .find(|entry| entry.id == file);

                if let Some(entry) = entry {
                    entry.status = FileEntryStatus::Ready;
                } else {
                    let mut entry = generate_file_entry(id.to_string(), &target, &junkyard.path);
                    entry.status = FileEntryStatus::Ready;
                    transaction.entries.push(entry);
                }
            }

            None
        } else if let Some(index) = junkyard.trashed.iter().position(|entry| entry.id == id) {
            let transaction = &mut junkyard.trashed[index];
            let entry = transaction
                .entries
                .iter_mut()
                .find(|entry| entry.id == file);

            if let Some(entry) = entry {
                entry.status = FileEntryStatus::Ready;
            } else {
                let mut entry = generate_file_entry(id.to_string(), &target, &junkyard.path);
                entry.status = FileEntryStatus::Ready;
                transaction.entries.push(entry);
            }

            None
        } else {
            let mut entry = generate_file_entry(id.to_string(), &target, &junkyard.path);
            entry.status = FileEntryStatus::Ready;

            junkyard.trashed.push(FileTransaction {
                id: id.to_owned(),
                entries: vec![entry],
            });

            junkyard
                .trashed
                .sort_unstable_by_key(|entry| Reverse(entry.id.clone()));

            if junkyard.trashed.len() > 9 {
                junkyard.trashed.pop()
            } else {
                None
            }
        }
    } else {
        None
    }
}

fn generate_file_entry(id: String, path: &Path, cache: &Path) -> FileEntry {
    let id = compose_compression_name(id, path);

    FileEntry {
        cache: cache.join(&id),
        id,
        status: FileEntryStatus::default(),
        target: path.to_path_buf(),
    }
}

fn compose_compression_name(id: String, path: &Path) -> String {
    let path = path
        .to_string_lossy()
        .replace('%', "%0025%")
        .replace('/', "%002F%")
        .replace('\\', "%005C%")
        .replace(':', "%003A%");

    let file_name = format!("{}%{}", id, path);

    file_name
}

fn decompose_compression_path(path: &Path) -> Option<(String, String, PathBuf)> {
    if let Some(file_name) = path.file_name() {
        let file_name = file_name.to_string_lossy();
        if let Some((id, path_string)) = file_name.split_once('%') {
            let path = path_string
                .replace("%002F%", "/")
                .replace("%005C%", "\\")
                .replace("%003A%", ":")
                .replace("%0025%", "%");

            Some((id.to_string(), file_name.to_string(), PathBuf::from(path)))
        } else {
            None
        }
    } else {
        None
    }
}

pub fn paste(junk: &JunkYard, buffer: &FileTreeBuffer, entry_id: &char) -> Vec<Action> {
    if let Some(transaction) = get_junkyard_transaction(junk, entry_id) {
        let mut actions = Vec::new();
        for entry in transaction.entries.iter() {
            actions.push(Action::Task(Task::RestorePath(
                entry.clone(),
                buffer.current.path.clone(),
            )));
        }
        actions
    } else {
        Vec::new()
    }
}

pub fn yank(junk: &mut JunkYard, buffer: &FileTreeBuffer, repeat: &usize) -> Vec<Action> {
    let current_buffer = &buffer.current.buffer;
    if current_buffer.lines.is_empty() {
        Vec::new()
    } else if let Some(cursor) = &buffer.current_cursor {
        let mut paths = Vec::new();
        for rpt in 0..*repeat {
            let line_index = cursor.vertical_index + rpt;
            if let Some(line) = current_buffer.lines.get(line_index) {
                let target = buffer.current.path.join(line.content.to_stripped_string());

                paths.push(target);
            }
        }

        let mut actions = Vec::new();
        let (transaction, obsolete) = yank_path(junk, paths);
        for entry in transaction.entries {
            actions.push(Action::Task(Task::YankPath(entry)));
        }

        if let Some(obsolete) = obsolete {
            for entry in obsolete.entries {
                actions.push(Action::Task(Task::DeleteJunkYardEntry(entry)));
            }
        }

        actions
    } else {
        Vec::new()
    }
}

fn yank_path(
    junkyard: &mut JunkYard,
    paths: Vec<PathBuf>,
) -> (FileTransaction, Option<FileTransaction>) {
    junkyard.current = FileEntryType::Yank;

    let transaction = generate_file_transaction(paths, junkyard);
    (transaction.clone(), junkyard.yanked.replace(transaction))
}

fn generate_file_transaction(paths: Vec<PathBuf>, junk: &JunkYard) -> FileTransaction {
    let added_at = match time::SystemTime::now().duration_since(time::UNIX_EPOCH) {
        Ok(time) => time.as_millis(),
        Err(_) => 0,
    };

    let entries = paths
        .into_iter()
        .map(|path| generate_file_entry(added_at.to_string(), &path, &junk.path))
        .collect();

    FileTransaction {
        id: added_at.to_string(),
        entries,
    }
}

pub fn get_junkyard_transaction<'a>(
    junkyard: &'a JunkYard,
    junk: &char,
) -> Option<&'a FileTransaction> {
    let transaction = match junk {
        '"' => match junkyard.current {
            FileEntryType::Trash => junkyard.trashed.first(),
            FileEntryType::Yank => junkyard.yanked.as_ref(),
            FileEntryType::_Custom(_) => None,
        },
        '0' => junkyard.yanked.as_ref(),
        '1' => junkyard.trashed.first(),
        '2' => junkyard.trashed.get(1),
        '3' => junkyard.trashed.get(2),
        '4' => junkyard.trashed.get(3),
        '5' => junkyard.trashed.get(4),
        '6' => junkyard.trashed.get(5),
        '7' => junkyard.trashed.get(6),
        '8' => junkyard.trashed.get(7),
        '9' => junkyard.trashed.get(8),
        // TODO: add custom junk handling
        _ => None,
    };

    let is_ready = transaction.as_ref().is_some_and(|trnsctn| {
        trnsctn
            .entries
            .iter()
            .all(|entry| entry.status == FileEntryStatus::Ready)
    });

    if is_ready {
        transaction
    } else {
        None
    }
}

pub fn remove_from_junkyard(junkyard: &mut JunkYard, path: &Path) {
    if let Some((id, _, _)) = decompose_compression_path(path) {
        let index = junkyard.trashed.iter().position(|entry| entry.id == id);
        if let Some(index) = index {
            junkyard.trashed.remove(index);
        }
    }
}

pub fn trash_to_junkyard(
    junkyard: &mut JunkYard,
    paths: Vec<PathBuf>,
) -> (FileTransaction, Option<FileTransaction>) {
    junkyard.current = FileEntryType::Trash;

    let transaction = generate_file_transaction(paths, junkyard);
    junkyard.trashed.insert(0, transaction.clone());

    let obsolete = if junkyard.trashed.len() > 9 {
        junkyard.trashed.pop()
    } else {
        None
    };

    (transaction, obsolete)
}

mod test {
    #[test]
    fn junk_add_or_update() {
        use std::path::PathBuf;

        use crate::update::junkyard::{add_or_update_junkyard_entry, trash_to_junkyard};

        let mut junk = super::JunkYard {
            current: Default::default(),
            path: std::path::PathBuf::from("/some/path"),
            trashed: Vec::new(),
            yanked: None,
        };

        let path = PathBuf::from("/other/path/.direnv");
        let (transaction, _) = trash_to_junkyard(&mut junk, vec![path]);

        assert_eq!(1, junk.trashed.len());
        assert_eq!(
            super::FileEntryStatus::Processing,
            transaction.entries[0].status
        );

        let id = transaction.id + "%%002F%other%002F%path%002F%.direnv";
        let path = PathBuf::from("/some/path").join(id);
        add_or_update_junkyard_entry(&mut junk, &path);

        assert_eq!(1, junk.trashed.len());
        assert_eq!(
            super::FileEntryStatus::Ready,
            junk.trashed[0].entries[0].status
        );

        let transaction = "1708576379595";
        let id = transaction.to_owned() + "%%002F%home%002F%user%002F%src%002F%yeet%002F%.direnv";
        let path = PathBuf::from("/some/path").join(id);
        add_or_update_junkyard_entry(&mut junk, &path);

        assert_eq!(2, junk.trashed.len());
        assert_eq!(transaction, junk.trashed[1].id);
        assert_eq!(
            super::FileEntryStatus::Ready,
            junk.trashed[1].entries[0].status
        );

        let file = "%%002F%home%002F%user%002F%src%002F%yeet%002F%awesome".to_owned();
        let id = transaction.to_owned() + &file;
        let path = PathBuf::from("/some/path").join(&id);
        add_or_update_junkyard_entry(&mut junk, &path);

        assert_eq!(2, junk.trashed.len());
        assert_eq!(transaction, junk.trashed[1].id);
        assert_eq!(id, junk.trashed[1].entries[1].id);
        assert_eq!(
            super::FileEntryStatus::Ready,
            junk.trashed[1].entries[1].status
        );

        let id_new = "2708576379595%%002F%home%002F%user%002F%src%002F%yeet%002F%.direnv";
        let path = PathBuf::from("/some/path").join(id_new);
        add_or_update_junkyard_entry(&mut junk, &path);

        assert_eq!(3, junk.trashed.len());
        assert_eq!(transaction, junk.trashed[2].id);
    }

    #[test]
    fn compose_decompose_compression_name_linux() {
        let id = "1708576379595".to_string();

        let path = std::path::Path::new("/home/U0025/sr%/y%et/%direnv");
        let name = super::compose_compression_name(id.to_string(), path);

        let composed = std::path::Path::new("/some/cache/junk/").join(name.clone());
        let (dec_id, dec_name, dec_path) = super::decompose_compression_path(&composed).unwrap();

        assert_eq!(id.to_string(), dec_id);
        assert_eq!(name, dec_name);
        assert!(!name.contains("/"));
        assert_eq!(path, dec_path);

        let path = std::path::Path::new("/home/user/sr%/y%et/%direnv");
        let name = super::compose_compression_name(id.to_string(), path);

        let composed = std::path::Path::new("/some/cache/junk/").join(name.clone());
        let (dec_id, dec_name, dec_path) = super::decompose_compression_path(&composed).unwrap();

        assert_eq!(id.to_string(), dec_id);
        assert_eq!(name, dec_name);
        assert!(!name.contains("/"));
        assert_eq!(path, dec_path);

        let path = std::path::Path::new("/home/user/src/yeet/.direnv");
        let name = super::compose_compression_name(id.to_string(), path);

        let composed = std::path::Path::new("/some/cache/junk/").join(name.clone());
        let (dec_id, dec_name, dec_path) = super::decompose_compression_path(&composed).unwrap();

        assert_eq!(id.to_string(), dec_id);
        assert_eq!(name, dec_name);
        assert!(!name.contains("/"));
        assert_eq!(path, dec_path);
    }

    #[test]
    fn compose_decompose_compression_name_windows() {
        let id = "1708576379595".to_string();

        let path = std::path::Path::new("c:\\home\\U0025\\sr%\\y%et\\%direnv");
        let name = super::compose_compression_name(id.to_string(), path);

        let composed = std::path::Path::new("c:\\some\\cache\\junk\\").join(name.clone());
        let (dec_id, dec_name, dec_path) = super::decompose_compression_path(&composed).unwrap();

        assert_eq!(id.to_string(), dec_id);
        assert_eq!(name, dec_name);
        assert!(!name.contains("\\") && !name.contains(":"));
        assert_eq!(path, dec_path);

        let path = std::path::Path::new("c:\\home\\user\\sr%\\y%et\\%direnv");
        let name = super::compose_compression_name(id.to_string(), path);

        let composed = std::path::Path::new("c:\\some\\cache\\junk\\").join(name.clone());
        let (dec_id, dec_name, dec_path) = super::decompose_compression_path(&composed).unwrap();

        assert_eq!(id.to_string(), dec_id);
        assert_eq!(name, dec_name);
        assert!(!name.contains("\\") && !name.contains(":"));
        assert_eq!(path, dec_path);

        let path = std::path::Path::new("c:\\home\\user\\src\\yeet\\.direnv");
        let name = super::compose_compression_name(id.to_string(), path);

        let composed = std::path::Path::new("c:\\some\\cache\\junk\\").join(name.clone());
        let (dec_id, dec_name, dec_path) = super::decompose_compression_path(&composed).unwrap();

        assert_eq!(id.to_string(), dec_id);
        assert_eq!(name, dec_name);
        assert!(!name.contains("\\") && !name.contains(":"));
        assert_eq!(path, dec_path);
    }
}
