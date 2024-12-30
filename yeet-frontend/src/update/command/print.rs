use yeet_keymap::message::{KeymapMessage, PrintContent};

use crate::{
    action::{self, Action},
    model::{
        junkyard::{FileEntryStatus, FileTransaction, JunkYard},
        mark::Marks,
        qfix::QuickFix,
        register::Register,
        Tasks,
    },
    update::junkyard::get_junkyard_transaction,
};

pub fn marks(marks: &Marks) -> Vec<Action> {
    let mut marks: Vec<_> = marks
        .entries
        .iter()
        .map(|(key, path)| (key, path.to_string_lossy().to_string()))
        .map(|(key, path)| format!("{:<4} {}", key, path))
        .collect();

    marks.sort();

    let mut contents = vec![":marks".to_string(), "Char Content".to_string()];
    contents.extend(marks);

    let content = contents
        .iter()
        .map(|cntnt| PrintContent::Default(cntnt.to_string()))
        .collect();

    vec![action::emit_keymap(KeymapMessage::Print(content))]
}

pub fn tasks(tasks: &Tasks) -> Vec<Action> {
    let mut contents = vec![":tl".to_string(), "Id   Task".to_string()];
    let mut tasks: Vec<_> = tasks
        .running
        .values()
        .map(|task| format!("{:<4} {}", task.id, task.external_id))
        .collect();

    tasks.sort();
    contents.extend(tasks);

    let content = contents
        .iter()
        .map(|cntnt| PrintContent::Default(cntnt.to_string()))
        .collect();

    vec![action::emit_keymap(KeymapMessage::Print(content))]
}

pub fn qfix(qfix: &QuickFix) -> Vec<Action> {
    let max_width = (qfix.entries.len() + 1).to_string().len();

    let entries: Vec<_> = qfix
        .entries
        .iter()
        .enumerate()
        .map(|(i, path)| (i + 1, path.to_string_lossy().to_string()))
        .map(|(i, path)| format!("{:>max_width$} {}", i, path))
        .collect();

    let mut contents = vec![":cl".to_string()];
    if entries.is_empty() {
        contents.push("no entries".to_string());
    } else {
        contents.extend(entries);
    }

    let content = contents
        .iter()
        .enumerate()
        .map(|(i, cntnt)| {
            if i == qfix.current_index + 1 {
                PrintContent::Information(cntnt.to_string())
            } else {
                PrintContent::Default(cntnt.to_string())
            }
        })
        .collect();

    vec![action::emit_keymap(KeymapMessage::Print(content))]
}

pub fn junkyard(junkyard: &JunkYard) -> Vec<Action> {
    let mut contents = vec![":junk".to_string(), "Name Content".to_string()];
    if let Some(current) = get_junkyard_transaction(junkyard, &'"') {
        contents.push(print_junkyard_entry("\"\"", current));
    }
    if let Some(yanked) = &junkyard.yanked {
        contents.push(print_junkyard_entry("\"0", yanked));
    }
    for (index, entry) in junkyard.trashed.iter().enumerate() {
        let junk_name = format!("\"{}", index + 1);
        contents.push(print_junkyard_entry(&junk_name, entry));
    }

    let content = contents
        .iter()
        .map(|cntnt| PrintContent::Default(cntnt.to_string()))
        .collect();

    vec![action::emit_keymap(KeymapMessage::Print(content))]
}

fn print_junkyard_entry(junk: &str, transaction: &FileTransaction) -> String {
    let is_ready = transaction
        .entries
        .iter()
        .all(|entry| entry.status == FileEntryStatus::Ready);

    let content = if is_ready {
        transaction
            .entries
            .iter()
            .map(|entry| entry.target.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        "Processing".to_string()
    };

    format!("{:<4} {}", junk, content)
}

pub fn register(register: &Register) -> Vec<Action> {
    let mut contents = vec![":reg".to_string(), "Name Content".to_string()];

    for (key, content) in register.content.iter() {
        contents.push(print_content(key, content));
    }

    if let Some(last_macro) = &register.last_macro {
        contents.push(print_content(&'@', last_macro));
    }
    if let Some(dot) = &register.dot {
        contents.push(print_content(&'.', dot));
    }
    if let Some(command) = &register.command {
        contents.push(print_content(&':', command));
    }
    if let Some(searched) = &register.searched {
        contents.push(print_content(&'/', &searched.1));
    }

    let content = contents
        .iter()
        .map(|cntnt| PrintContent::Default(cntnt.to_string()))
        .collect();

    vec![action::emit_keymap(KeymapMessage::Print(content))]
}

fn print_content(prefix: &char, content: &str) -> String {
    format!("\"{:<3} {}", prefix, content)
}
