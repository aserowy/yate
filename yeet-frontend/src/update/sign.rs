use std::path::{Path, PathBuf};
use yeet_buffer::model::{BufferLine, Sign, SignIdentifier};

use crate::model::{
    mark::{Marks, MARK_SIGN_ID},
    qfix::{QuickFix, QFIX_SIGN_ID},
    Buffer,
};

pub fn set_sign_if_qfix(qfix: &QuickFix, bl: &mut BufferLine, path: &Path) {
    let is_marked = qfix.entries.iter().any(|p| p == path);
    if !is_marked {
        return;
    }

    set(bl, QFIX_SIGN_ID);
}

pub fn set_sign_if_marked(marks: &Marks, bl: &mut BufferLine, path: &Path) {
    let is_marked = marks.entries.values().any(|p| p == path);
    if !is_marked {
        return;
    }

    set(bl, MARK_SIGN_ID);
}

pub fn set(bl: &mut BufferLine, sign_id: SignIdentifier) {
    let is_signed = bl.signs.iter().any(|s| s.id == sign_id);
    if is_signed {
        return;
    }

    if let Some(sign) = generate_sign(sign_id) {
        bl.signs.push(sign);
    }
}

pub fn set_sign_for_paths(buffers: Vec<&mut Buffer>, paths: Vec<PathBuf>, sign_id: SignIdentifier) {
    for buffer in buffers {
        let buffer = match buffer {
            Buffer::FileTree(it) => it,
            _ => continue,
        };

        for path in &paths {
            let parent = match path.parent() {
                Some(it) => it,
                None => return,
            };

            let target = buffer
                .get_mut_directories()
                .into_iter()
                .find_map(|(p, _, _, b)| if p == parent { Some(b) } else { None });

            let buffer = match target {
                Some(buffer) => buffer,
                None => return,
            };

            let file_name = match path.file_name() {
                Some(it) => match it.to_str() {
                    Some(it) => it,
                    None => return,
                },
                None => return,
            };

            if let Some(line) = buffer
                .lines
                .iter_mut()
                .find(|bl| bl.content.to_stripped_string() == file_name)
            {
                set(line, sign_id);
            }
        }
    }
}

fn generate_sign(sign_id: SignIdentifier) -> Option<Sign> {
    match sign_id {
        QFIX_SIGN_ID => Some(Sign {
            id: QFIX_SIGN_ID,
            content: 'c',
            style: "\x1b[1;95m".to_string(),
            priority: 0,
        }),
        MARK_SIGN_ID => Some(Sign {
            id: MARK_SIGN_ID,
            content: 'm',
            style: "\x1b[1;96m".to_string(),
            priority: 0,
        }),
        _ => None,
    }
}

pub fn unset_sign_on_all_buffers(buffers: Vec<&mut Buffer>, sign_id: SignIdentifier) {
    for buffer in buffers {
        let buffer = match buffer {
            Buffer::FileTree(it) => it,
            _ => continue,
        };

        buffer
            .get_mut_directories()
            .into_iter()
            .flat_map(|(_, _, _, b)| &mut b.lines)
            .for_each(|l| unset(l, sign_id));
    }
}

pub fn unset_sign_for_paths(
    buffers: Vec<&mut Buffer>,
    paths: Vec<PathBuf>,
    sign_id: SignIdentifier,
) {
    for buffer in buffers {
        let buffer = match buffer {
            Buffer::FileTree(it) => it,
            _ => continue,
        };

        for path in &paths {
            let parent = match path.parent() {
                Some(it) => it,
                None => return,
            };

            let target = buffer
                .get_mut_directories()
                .into_iter()
                .find_map(|(p, _, _, b)| if p == parent { Some(b) } else { None });

            let buffer = match target {
                Some(buffer) => buffer,
                None => return,
            };

            let file_name = match path.file_name() {
                Some(it) => match it.to_str() {
                    Some(it) => it,
                    None => return,
                },
                None => return,
            };

            if let Some(line) = buffer
                .lines
                .iter_mut()
                .find(|bl| bl.content.to_stripped_string() == file_name)
            {
                unset(line, sign_id);
            }
        }
    }
}

pub fn unset(bl: &mut BufferLine, sign_id: SignIdentifier) {
    let position = bl.signs.iter().position(|s| s.id == sign_id);
    if let Some(position) = position {
        bl.signs.remove(position);
    }
}
