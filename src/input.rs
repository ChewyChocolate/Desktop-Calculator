use crate::app::CalculatorApp;
use eframe::egui::{self, Event, Key};

pub fn handle(app: &mut CalculatorApp, ctx: &egui::Context) {
    let editing = ctx.memory(|m| m.has_focus(egui::Id::new("expression_editor")));
    let events = ctx.input(|i| i.events.clone());
    let editor_id = egui::Id::new("expression_editor");
    let at_end = egui::TextEdit::load_state(ctx, editor_id)
        .and_then(|state| state.cursor.char_range())
        .is_some_and(|range| {
            range.is_empty() && range.primary.index == app.expression.chars().count()
        });
    let explicit_edit = events.iter().any(|event| {
        matches!(
            event,
            Event::PointerButton { pressed: true, .. }
                | Event::Key {
                    pressed: true,
                    key: Key::ArrowLeft | Key::ArrowRight | Key::Home | Key::End,
                    ..
                }
        ) || matches!(event, Event::Key { key: Key::A, pressed: true, modifiers, .. } if modifiers.command)
    });
    if editing
        && explicit_edit
        && !events
            .iter()
            .any(|event| matches!(event, Event::PointerButton { pressed: true, .. }))
    {
        app.begin_edit();
    }
    let mut consumed_text = Vec::new();
    for (index, event) in events.into_iter().enumerate() {
        match event {
            Event::Copy if !editing => app.copy(ctx),
            Event::Paste(text) if !editing => app.paste(&text),
            Event::Key {
                key,
                pressed: true,
                modifiers,
                ..
            } => {
                if modifiers.ctrl && matches!(key, Key::Num1 | Key::Num2) {
                    app.settings.scientific = key == Key::Num2;
                    continue;
                }
                if key == Key::H && modifiers.ctrl {
                    app.settings.history_open = !app.settings.history_open;
                    continue;
                }
                if modifiers.command {
                    if key == Key::C && !editing {
                        app.copy(ctx);
                    }
                    continue;
                }
                match key {
                    Key::Enter if editing => app.evaluate_requested = true,
                    Key::Enter => app.evaluate(),
                    Key::Escape => {
                        if app.help_open {
                            app.help_open = false;
                        } else if app.settings.history_open
                            && ctx.content_rect().width() < crate::ui::HISTORY_BREAKPOINT
                        {
                            app.settings.history_open = false;
                        } else {
                            app.clear();
                        }
                    }
                    Key::Backspace if !editing => app.backspace(),
                    Key::Delete if !editing => app.clear_entry(),
                    Key::F1 => app.help_open = !app.help_open,
                    _ => {}
                }
            }
            Event::Text(text)
                if editing && app.evaluated && at_end && !explicit_edit && text != "=" =>
            {
                let text = crate::app::normalize_expression(&text);
                // Match calculator input while keeping caret/selection edits native.
                for c in text.chars() {
                    app.insert(&c.to_string());
                }
                consumed_text.push(index);
                if let Some(mut state) = egui::TextEdit::load_state(ctx, editor_id) {
                    state
                        .cursor
                        .set_char_range(Some(egui::text::CCursorRange::one(
                            egui::text::CCursor::new(app.expression.chars().count()),
                        )));
                    state.store(ctx, editor_id);
                }
            }
            Event::Text(text) if editing && text == "=" => {
                app.evaluate_requested = true;
            }
            Event::Text(text) if !editing && !app.help_open => {
                for c in text.chars() {
                    match c {
                        '=' => app.evaluate(),
                        '0'..='9'
                        | '.'
                        | '+'
                        | '-'
                        | '*'
                        | '/'
                        | '('
                        | ')'
                        | '^'
                        | '%'
                        | '!'
                        | 'a'..='z'
                        | 'A'..='Z' => app.insert(&c.to_string()),
                        '−' => app.insert("-"),
                        '×' => app.insert("*"),
                        '÷' => app.insert("/"),
                        'π' => app.insert("pi"),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    // Enter always evaluates, even when a keypad button has keyboard focus.
    ctx.input_mut(|i| {
        let mut index = 0;
        i.events.retain(|_| {
            let keep = !consumed_text.contains(&index);
            index += 1;
            keep
        });
        i.consume_key(egui::Modifiers::NONE, Key::Enter);
        i.consume_key(egui::Modifiers::NONE, Key::Escape);
        if editing {
            i.events
                .retain(|event| !matches!(event, Event::Text(text) if text == "="));
        }
    });
}
