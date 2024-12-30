use yeet_buffer::model::SignIdentifier;

use crate::model::{mark::MARK_SIGN_ID, qfix::QFIX_SIGN_ID, Buffer, FileTreeBuffer, Model};

pub fn update(model: &mut Model) {
    let settings = &model.settings;

    for buffer in model.app.buffers.values_mut() {
        let buffer = match buffer {
            Buffer::FileTree(it) => it,
            Buffer::_Text(_) => todo!(),
        };

        buffer.current_vp.set(&settings.current);
        buffer.parent_vp.set(&settings.current);
        buffer.preview_vp.set(&settings.current);

        if settings.show_mark_signs {
            remove_hidden_sign_on_all_buffer(buffer, &MARK_SIGN_ID);
        } else {
            add_hidden_sign_on_all_buffer(buffer, MARK_SIGN_ID);
        }

        if settings.show_quickfix_signs {
            remove_hidden_sign_on_all_buffer(buffer, &QFIX_SIGN_ID);
        } else {
            add_hidden_sign_on_all_buffer(buffer, QFIX_SIGN_ID);
        }
    }
}

fn add_hidden_sign_on_all_buffer(buffer: &mut FileTreeBuffer, id: SignIdentifier) {
    buffer.current_vp.hidden_sign_ids.insert(id);
    buffer.parent_vp.hidden_sign_ids.insert(id);
    buffer.preview_vp.hidden_sign_ids.insert(id);
}

fn remove_hidden_sign_on_all_buffer(buffer: &mut FileTreeBuffer, id: &SignIdentifier) {
    buffer.current_vp.hidden_sign_ids.remove(id);
    buffer.parent_vp.hidden_sign_ids.remove(id);
    buffer.preview_vp.hidden_sign_ids.remove(id);
}
