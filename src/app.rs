use crate::{
    calculator::{self, Angle, Value},
    history::{Entry, History},
    settings::Settings,
};
use eframe::egui;
use std::time::{Duration, Instant};

pub struct CalculatorApp {
    pub settings: Settings,
    pub history: History,
    pub expression: String,
    pub result: String,
    pub error: Option<String>,
    pub evaluated: bool,
    pub approximate: bool,
    pub help_open: bool,
    pub notice: Option<(String, Instant)>,
    pub result_valid: bool,
    pub evaluate_requested: bool,
    pub cleared_history: Option<Vec<Entry>>,
    approximate_input: bool,
    repeat: Option<(char, Value)>,
    value: Option<Value>,
}
impl CalculatorApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        crate::ui::install_fonts(&cc.egui_ctx);
        let settings = cc
            .storage
            .and_then(|s| eframe::get_value(s, "settings"))
            .unwrap_or_default();
        let mut history: History = cc
            .storage
            .and_then(|s| eframe::get_value(s, "history"))
            .unwrap_or_default();
        history.sanitize();
        Self {
            settings,
            history,
            expression: String::new(),
            result: "0".into(),
            error: None,
            evaluated: false,
            approximate: false,
            help_open: false,
            notice: None,
            result_valid: false,
            evaluate_requested: false,
            cleared_history: None,
            approximate_input: false,
            repeat: None,
            value: None,
        }
    }
    pub fn notify(&mut self, message: &str) {
        self.notice = Some((message.into(), Instant::now()));
    }
    pub fn clear(&mut self) {
        self.expression.clear();
        self.result = "0".into();
        self.error = None;
        self.evaluated = false;
        self.repeat = None;
        self.value = None;
        self.approximate = false;
        self.approximate_input = false;
        self.result_valid = false;
        self.notice = None;
    }
    pub fn insert(&mut self, text: &str) {
        let operator = matches!(text, "+" | "-" | "*" | "/" | "^" | "%" | "!");
        if self.evaluated {
            if operator {
                self.start_from_result();
            } else {
                self.clear();
            }
        }
        if self.expression.len() + text.len() > 2048 {
            self.notify("Expression limit reached");
            return;
        }
        self.expression.push_str(text);
        self.error = None;
        self.repeat = None;
        self.preview();
    }
    pub fn edit_expression(&mut self, expression: String) {
        let expression = normalize_expression(&expression);
        if expression.len() > 2048 {
            self.notify("Expression limit reached");
            return;
        }
        if expression.is_empty() {
            self.approximate_input = false;
        }
        self.expression = expression;
        self.evaluated = false;
        self.error = None;
        self.notice = None;
        self.repeat = None;
        self.preview();
    }
    pub fn preview(&mut self) {
        self.result_valid = false;
        self.error = None;
        if self.expression.is_empty() {
            self.result = "0".into();
            self.approximate = false;
            self.approximate_input = false;
        } else {
            match calculator::evaluate(&self.expression, self.settings.angle) {
                Ok(e) => {
                    self.result = e.value.text();
                    self.approximate =
                        self.approximate_input || matches!(e.value, Value::Scientific(_));
                    self.result_valid = true;
                }
                Err(e) if e.is_incomplete() => {}
                Err(e) => self.error = Some(e.to_string()),
            }
        }
    }
    fn start_from_result(&mut self) {
        self.expression = self.result.clone();
        self.approximate_input = self.approximate;
        self.evaluated = false;
    }
    pub fn begin_edit(&mut self) {
        self.evaluated = false;
        self.repeat = None;
    }
    pub fn nth_root(&mut self) {
        if self.evaluated {
            self.start_from_result();
        }
        self.insert("^(1/");
    }
    pub fn clear_history(&mut self) {
        if !self.history.entries.is_empty() {
            self.cleared_history = Some(std::mem::take(&mut self.history.entries));
        }
    }
    pub fn restore_history(&mut self) {
        if let Some(mut entries) = self.cleared_history.take() {
            entries.append(&mut self.history.entries);
            self.history.entries = entries;
            self.history.sanitize();
            self.notify("History restored");
        }
    }
    pub fn evaluate(&mut self) {
        if self.expression.is_empty() {
            return;
        }
        let expression = if self.evaluated {
            if let (Some((op, rhs)), Some(value)) = (self.repeat, self.value) {
                let expression = format!("{}{}({})", value.text(), op, rhs.text());
                match calculator::binary(op, value, rhs) {
                    Ok(v) => {
                        self.commit(expression, v, self.repeat);
                    }
                    Err(e) => {
                        self.error = Some(e.to_string());
                        self.result_valid = false;
                    }
                }
                return;
            }
            return;
        } else {
            self.expression.clone()
        };
        match calculator::evaluate(&expression, self.settings.angle) {
            Ok(e) => self.commit(expression, e.value, e.repeat),
            Err(e) => {
                self.error = Some(e.to_string());
                self.result_valid = false;
            }
        }
    }
    fn commit(&mut self, expression: String, value: Value, repeat: Option<(char, Value)>) {
        self.result = value.text();
        self.approximate = self.approximate_input || matches!(value, Value::Scientific(_));
        self.history.push(Entry {
            expression: expression.clone(),
            result: self.result.clone(),
            approximate: self.approximate,
        });
        self.expression = expression;
        self.evaluated = true;
        self.value = Some(value);
        self.repeat = repeat;
        self.error = None;
        self.result_valid = true;
    }
    pub fn backspace(&mut self) {
        if self.evaluated {
            self.start_from_result();
        }
        self.expression.pop();
        self.error = None;
        self.repeat = None;
        self.preview();
    }
    pub fn clear_entry(&mut self) {
        if self.evaluated {
            self.clear();
            return;
        }
        let start = operand_start(&self.expression);
        self.expression.truncate(start);
        self.error = None;
        self.repeat = None;
        self.preview();
    }
    pub fn unary(&mut self, operation: &str) {
        if self.evaluated {
            self.start_from_result();
        }
        if self.expression.is_empty() {
            if matches!(
                operation,
                "sqrt"
                    | "cbrt"
                    | "sin"
                    | "cos"
                    | "tan"
                    | "asin"
                    | "acos"
                    | "atan"
                    | "log"
                    | "ln"
                    | "exp"
            ) {
                self.insert(&format!("{operation}("));
            }
            return;
        }
        let start = operand_start(&self.expression);
        let operand = self.expression[start..].to_owned();
        if operand.is_empty() {
            if operation == "sign" {
                self.insert("-");
            } else if !matches!(operation, "square" | "reciprocal") {
                self.insert(&format!("{operation}("));
            }
            return;
        }
        let replacement = match operation {
            "sign" => format!("-({operand})"),
            "square" => format!("({operand})^2"),
            "reciprocal" => format!("(1/({operand}))"),
            _ => format!("{operation}({operand})"),
        };
        if start + replacement.len() > 2048 {
            self.notify("Expression limit reached");
            return;
        }
        self.expression.truncate(start);
        self.expression.push_str(&replacement);
        self.error = None;
        self.repeat = None;
        self.preview();
    }
    pub fn reuse(&mut self, entry: &Entry) {
        self.clear();
        self.expression = entry.result.clone();
        self.approximate_input = entry.approximate;
        self.preview();
        if self.result_valid {
            self.value = calculator::evaluate(&self.expression, self.settings.angle)
                .ok()
                .map(|e| e.value);
            self.evaluated = true;
        }
        self.notify("Result recalled");
    }
    /// Restore the original editable expression alongside the saved result.
    pub fn restore_calculation(&mut self, entry: &Entry) {
        self.reuse(entry);
        self.expression = entry.expression.clone();
        self.notify("Calculation restored");
    }
    pub fn paste(&mut self, text: &str) {
        let text = normalize_expression(text.trim());
        if text.len() > 2048
            || text.is_empty()
            || calculator::evaluate(&text, self.settings.angle).is_err()
        {
            self.notify("Paste a valid number or expression");
            return;
        }
        // A pasted expression is one operand, preserving its grouping.
        if text.starts_with('-') || text.starts_with('+') || text.parse::<f64>().is_err() {
            self.insert(&format!("({text})"));
        } else {
            self.insert(&text);
        }
    }
    pub fn copy(&mut self, ctx: &egui::Context) {
        if !self.result_valid {
            self.notify("Complete a valid calculation before copying");
            return;
        }
        ctx.copy_text(self.result.clone());
        self.notify("Result copied");
    }
    pub fn toggle_angle(&mut self) {
        self.settings.angle = if self.settings.angle == Angle::Degrees {
            Angle::Radians
        } else {
            Angle::Degrees
        };
        self.evaluated = false;
        self.repeat = None;
        self.value = None;
        self.preview();
    }
}
pub fn normalize_expression(text: &str) -> String {
    text.replace('−', "-")
        .replace('×', "*")
        .replace('÷', "/")
        .replace('π', "pi")
}
// Locate the last complete operand, preserving scientific exponent signs.
fn operand_start(s: &str) -> usize {
    let mut depth = 0;
    for (i, c) in s.char_indices().rev() {
        match c {
            ')' => depth += 1,
            '(' if depth > 0 => depth -= 1,
            '+' | '-' | '*' | '/' | '^' if depth == 0 => {
                let previous = s[..i].chars().next_back();
                if matches!(c, '+' | '-')
                    && (previous.is_none()
                        || previous.is_some_and(|p| p == 'e' || p == 'E' || "+-*/^(".contains(p)))
                {
                    continue;
                }
                return i + c.len_utf8();
            }
            _ => {}
        }
    }
    0
}
impl eframe::App for CalculatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        crate::ui::apply_theme(ctx, self.settings.theme);
        crate::input::handle(self, ctx);
        crate::ui::show(self, ctx);
        if std::mem::take(&mut self.evaluate_requested) {
            self.evaluate();
            ctx.request_repaint();
        }
        if let Some((_, at)) = &self.notice {
            let remaining = Duration::from_secs(3).saturating_sub(at.elapsed());
            if remaining.is_zero() {
                self.notice = None;
            } else {
                ctx.request_repaint_after(remaining);
            }
        }
    }
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "settings", &self.settings);
        eframe::set_value(storage, "history", &self.history);
    }
    fn auto_save_interval(&self) -> Duration {
        Duration::from_secs(3)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn app() -> CalculatorApp {
        CalculatorApp {
            settings: Settings::default(),
            history: History::default(),
            expression: String::new(),
            result: "0".into(),
            error: None,
            evaluated: false,
            approximate: false,
            help_open: false,
            notice: None,
            result_valid: false,
            evaluate_requested: false,
            cleared_history: None,
            approximate_input: false,
            repeat: None,
            value: None,
        }
    }
    #[test]
    fn repeat_and_fresh_input() {
        let mut a = app();
        a.insert("2+3*4");
        a.evaluate();
        assert_eq!(a.result, "14");
        a.evaluate();
        assert_eq!(a.result, "26");
        a.insert("5");
        assert_eq!(a.expression, "5");
    }
    #[test]
    fn operand_operations() {
        let mut a = app();
        a.insert("10+9");
        a.unary("sqrt");
        a.evaluate();
        assert_eq!(a.result, "13");
        a.clear();
        a.insert("10+1e-3");
        a.clear_entry();
        assert_eq!(a.expression, "10+");
    }
    #[test]
    fn paste_validation() {
        let mut a = app();
        a.paste("1+2");
        assert_eq!(a.result, "3");
        a.clear();
        a.insert("2*");
        a.paste("-4");
        a.evaluate();
        assert_eq!(a.result, "-8");
        a.clear();
        a.insert("2*");
        a.paste("1+2");
        a.evaluate();
        assert_eq!(a.result, "6");
        a.clear();
        a.paste("not a calculation");
        assert!(a.expression.is_empty());
    }
    #[test]
    fn history_bound() {
        let mut h = History::default();
        for i in 0..600 {
            h.push(Entry {
                expression: i.to_string(),
                result: i.to_string(),
                approximate: false,
            });
        }
        assert_eq!(h.entries.len(), 500);
    }
    #[test]
    fn shifted_plus_and_equals_keyboard() {
        let mut a = app();
        let ctx = egui::Context::default();
        let key = |key, modifiers| egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        };
        let raw = egui::RawInput {
            events: vec![
                egui::Event::Text("0.1".into()),
                key(egui::Key::Equals, egui::Modifiers::SHIFT),
                egui::Event::Text("+".into()),
                egui::Event::Text("0.2".into()),
                key(egui::Key::Enter, egui::Modifiers::NONE),
            ],
            ..Default::default()
        };
        let _ = ctx.run(raw, |ctx| crate::input::handle(&mut a, ctx));
        assert_eq!(a.expression, "0.1+0.2");
        assert_eq!(a.result, "0.3");
        assert_eq!(a.history.entries.len(), 1);
        let _ = ctx.run(
            egui::RawInput {
                events: vec![egui::Event::Text("=".into())],
                ..Default::default()
            },
            |ctx| crate::input::handle(&mut a, ctx),
        );
        assert_eq!(a.result, "0.5");
    }
    #[test]
    fn saved_state_round_trip() {
        use std::collections::HashMap;
        #[derive(Default)]
        struct Memory(HashMap<String, String>);
        impl eframe::Storage for Memory {
            fn get_string(&self, key: &str) -> Option<String> {
                self.0.get(key).cloned()
            }
            fn set_string(&mut self, key: &str, value: String) {
                self.0.insert(key.into(), value);
            }
            fn flush(&mut self) {}
        }
        let mut a = app();
        a.insert("0.1+0.2");
        a.evaluate();
        a.settings.theme = crate::settings::Theme::Dark;
        let mut storage = Memory::default();
        eframe::App::save(&mut a, &mut storage);
        let h: History = eframe::get_value(&storage, "history").unwrap();
        let settings: Settings = eframe::get_value(&storage, "settings").unwrap();
        assert_eq!(h.entries[0].result, "0.3");
        assert!(settings.theme == crate::settings::Theme::Dark);
    }
    #[test]
    fn keypad_fits_window() {
        for (w, h, history, scientific) in [
            (400.0, 660.0, false, false),
            (400.0, 660.0, false, true),
            (680.0, 660.0, true, true),
            (780.0, 740.0, true, false),
            (780.0, 740.0, true, true),
            (1200.0, 900.0, true, false),
        ] {
            let mut a = app();
            a.settings.history_open = history;
            a.settings.scientific = scientific;
            let ctx = egui::Context::default();
            let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(w, h));
            // Run a settling frame before measuring auto-sized widgets.
            for _ in 0..2 {
                let output = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(rect),
                        ..Default::default()
                    },
                    |ctx| {
                        crate::ui::apply_theme(ctx, a.settings.theme);
                        crate::ui::show(&mut a, ctx);
                    },
                );
                let equals = output
                    .shapes
                    .iter()
                    .find_map(|shape| match &shape.shape {
                        egui::epaint::Shape::Text(text) if text.galley.text() == "=" => {
                            let rect = text.visual_bounding_rect();
                            assert!(shape.clip_rect.contains_rect(rect), "equals outside clip at {w}x{h}, scientific={scientific}: {rect:?}, clip={:?}", shape.clip_rect);
                            Some(rect)
                        }
                        _ => None,
                    })
                    .expect("equals button is rendered");
                assert!(
                    rect.contains_rect(equals),
                    "equals clipped at {w}x{h}: {equals:?}"
                );
                let key = output
                    .shapes
                    .iter()
                    .find_map(|shape| match &shape.shape {
                        egui::epaint::Shape::Rect(key)
                            if key.rect.contains(equals.center())
                                && key.rect.height() >= 38.0
                                && key.rect.height() <= 80.0 =>
                        {
                            Some((key.rect, shape.clip_rect))
                        }
                        _ => None,
                    })
                    .expect("equals key has a visible background");
                assert!(
                    key.1.contains_rect(key.0),
                    "equals button clipped at {w}x{h}: {key:?}"
                );
            }
        }
    }
    fn editor_frame(
        a: &mut CalculatorApp,
        ctx: &egui::Context,
        events: Vec<egui::Event>,
    ) -> egui::FullOutput {
        ctx.run(
            egui::RawInput {
                time: Some(ctx.input(|i| i.time) + 1.1),
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(400.0, 660.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| {
                crate::ui::apply_theme(ctx, a.settings.theme);
                crate::input::handle(a, ctx);
                crate::ui::show(a, ctx);
                if std::mem::take(&mut a.evaluate_requested) {
                    a.evaluate();
                }
            },
        )
    }
    fn key(key: egui::Key, modifiers: egui::Modifiers) -> egui::Event {
        egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        }
    }
    #[test]
    fn invalid_preview_cannot_copy_stale_result() {
        let mut a = app();
        a.insert("2+3");
        assert!(a.result_valid);
        a.insert("/");
        assert!(!a.result_valid);
        assert_eq!(a.result, "5");
        a.evaluate();
        assert!(a.error.is_some());
        let ctx = egui::Context::default();
        let output = ctx.run(egui::RawInput::default(), |ctx| a.copy(ctx));
        assert!(output.platform_output.commands.is_empty());
        a.insert("2");
        assert!(a.result_valid);
        assert!(a.error.is_none());
        assert_eq!(a.result, "3.5");
    }
    #[test]
    fn editor_selection_paste_enter_and_middle_correction() {
        let mut a = app();
        let ctx = egui::Context::default();
        editor_frame(&mut a, &ctx, vec![]);
        ctx.memory_mut(|m| m.request_focus(egui::Id::new("expression_editor")));
        editor_frame(
            &mut a,
            &ctx,
            vec![
                egui::Event::Paste("sin(30)+2".into()),
                key(egui::Key::Enter, egui::Modifiers::NONE),
            ],
        );
        assert_eq!(a.expression, "sin(30)+2");
        assert_eq!(a.result, "2.5");
        assert!(a.evaluated);
        assert_eq!(a.history.entries.len(), 1);
        editor_frame(
            &mut a,
            &ctx,
            vec![
                key(egui::Key::A, egui::Modifiers::COMMAND),
                egui::Event::Paste("12+3".into()),
            ],
        );
        assert_eq!(a.expression, "12+3");
        editor_frame(
            &mut a,
            &ctx,
            vec![
                key(egui::Key::Home, egui::Modifiers::NONE),
                key(egui::Key::Delete, egui::Modifiers::NONE),
                egui::Event::Text("2".into()),
            ],
        );
        assert_eq!(a.expression, "22+3");
        assert_eq!(a.result, "25");
        assert!(!a.evaluated);
        editor_frame(&mut a, &ctx, vec![egui::Event::Text("=".into())]);
        assert_eq!(a.expression, "22+3");
        assert!(a.evaluated);
    }
    #[test]
    fn editor_undo_and_selection_copy() {
        let mut a = app();
        let ctx = egui::Context::default();
        editor_frame(&mut a, &ctx, vec![]);
        ctx.memory_mut(|m| m.request_focus(egui::Id::new("expression_editor")));
        editor_frame(&mut a, &ctx, vec![egui::Event::Text("2+3".into())]);
        let output = editor_frame(
            &mut a,
            &ctx,
            vec![
                key(egui::Key::A, egui::Modifiers::COMMAND),
                egui::Event::Copy,
            ],
        );
        assert!(
            output
                .platform_output
                .commands
                .iter()
                .any(|c| matches!(c, egui::OutputCommand::CopyText(text) if text == "2+3"))
        );
        editor_frame(&mut a, &ctx, vec![egui::Event::Paste("8*9".into())]);
        assert_eq!(a.result, "72");
        editor_frame(
            &mut a,
            &ctx,
            vec![key(egui::Key::Z, egui::Modifiers::COMMAND)],
        );
        assert_eq!(a.expression, "2+3");
        assert_eq!(a.result, "5");
    }
    #[test]
    fn history_restores_editable_calculation_and_approximation() {
        let mut a = app();
        a.edit_expression("sin(30)+2".into());
        a.evaluate();
        let entry = a.history.entries[0].clone();
        a.clear();
        a.restore_calculation(&entry);
        assert_eq!(a.expression, "sin(30)+2");
        assert_eq!(a.result, "2.5");
        assert!(a.approximate && a.result_valid);
        assert_eq!(a.history.entries.len(), 1);
        a.begin_edit();
        a.edit_expression("sin(30)+3".into());
        assert_eq!(a.result, "3.5");
    }

    #[test]
    fn mode_shortcuts_preserve_editor_input() {
        let mut a = app();
        let ctx = egui::Context::default();
        a.edit_expression("12+3".into());
        editor_frame(&mut a, &ctx, vec![]);
        ctx.memory_mut(|m| m.request_focus(egui::Id::new("expression_editor")));
        for (key_code, scientific) in [(egui::Key::Num2, true), (egui::Key::Num1, false)] {
            editor_frame(&mut a, &ctx, vec![key(key_code, egui::Modifiers::CTRL)]);
            assert_eq!(a.settings.scientific, scientific);
            assert_eq!(a.expression, "12+3");
            assert_eq!(a.result, "15");
        }
    }

    #[test]
    fn history_actions_fit_rail_and_restore_on_click() {
        for (width, history_open) in [(400.0, false), (720.0, true), (780.0, true), (1200.0, true)]
        {
            let ctx = egui::Context::default();
            crate::ui::install_fonts(&ctx);
            let mut a = app();
            a.settings.history_open = history_open;
            a.edit_expression("128*0.85".into());
            a.evaluate();
            a.clear();
            let mut run = |events| {
                ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(width, 660.0),
                        )),
                        events,
                        ..Default::default()
                    },
                    |ctx| {
                        crate::ui::apply_theme(ctx, a.settings.theme);
                        crate::ui::show(&mut a, ctx);
                    },
                )
            };
            run(vec![]);
            let output = run(vec![]);
            for shape in &output.shapes {
                if let egui::epaint::Shape::Text(text) = &shape.shape
                    && ["108.8", "128 × 0.85", "="].contains(&text.galley.text())
                {
                    assert!(
                        shape.clip_rect.contains_rect(text.visual_bounding_rect()),
                        "Clipped text at {width}: {text:?}"
                    );
                }
            }
            if history_open {
                let pos = output
                    .shapes
                    .iter()
                    .find_map(|shape| match &shape.shape {
                        egui::epaint::Shape::Text(text) if text.galley.text() == "108.8" => {
                            Some(text.visual_bounding_rect().center())
                        }
                        _ => None,
                    })
                    .unwrap();
                run(vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed: true,
                        modifiers: egui::Modifiers::NONE,
                    },
                ]);
                run(vec![egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::NONE,
                }]);
                assert_eq!(a.expression, "128*0.85");
                assert_eq!(a.result, "108.8");
            }
        }
    }

    #[test]
    fn history_copy_does_not_restore_or_change_the_current_calculation() {
        let mut a = app();
        a.settings.history_open = true;
        a.edit_expression("128*0.85".into());
        a.evaluate();
        a.edit_expression("7+2".into());
        let ctx = egui::Context::default();
        crate::ui::install_fonts(&ctx);
        let mut run = |events| {
            ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(780.0, 740.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    crate::ui::apply_theme(ctx, a.settings.theme);
                    crate::ui::show(&mut a, ctx);
                },
            )
        };
        run(vec![]);
        let output = run(vec![]);
        // Find the drawn copy icon beside the saved result in the right-hand rail.
        let pos = output
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::epaint::Shape::Rect(rect)
                    if rect.rect.width() == 9.0
                        && rect.rect.height() == 11.0
                        && rect.rect.center().x > 700.0
                        && rect.rect.center().y > 80.0 =>
                {
                    Some(rect.rect.center())
                }
                _ => None,
            })
            .expect("history copy icon is visible");
        run(vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ]);
        let copied = run(vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }]);
        assert!(
            copied
                .platform_output
                .commands
                .iter()
                .any(|c| matches!(c, egui::OutputCommand::CopyText(text) if text == "108.8"))
        );
        assert_eq!(a.expression, "7+2");
        assert_eq!(a.result, "9");
        assert_eq!(a.history.entries.len(), 1);
    }

    #[test]
    fn history_undo_preserves_new_calculations() {
        let mut a = app();
        a.insert("1+2");
        a.evaluate();
        a.clear_history();
        assert!(a.history.entries.is_empty());
        a.insert("4");
        a.evaluate();
        a.restore_history();
        assert_eq!(a.history.entries.len(), 2);
        assert_eq!(a.history.entries[0].result, "3");
        assert_eq!(a.history.entries[1].result, "4");
        assert!(a.cleared_history.is_none());
    }
    #[test]
    fn feedback_does_not_move_keypad() {
        for scientific in [false, true] {
            let mut a = app();
            a.settings.scientific = scientific;
            let ctx = egui::Context::default();
            let mut bounds = vec![];
            for state in 0..3 {
                if state == 1 {
                    a.notify("Result copied");
                }
                if state == 2 {
                    a.error = Some("Cannot divide by zero".into());
                }
                editor_frame(&mut a, &ctx, vec![]);
                let output = editor_frame(&mut a, &ctx, vec![]);
                bounds.push(
                    output
                        .shapes
                        .iter()
                        .find_map(|shape| match &shape.shape {
                            egui::epaint::Shape::Text(text) if text.galley.text() == "=" => {
                                Some(text.visual_bounding_rect())
                            }
                            _ => None,
                        })
                        .unwrap(),
                );
            }
            assert_eq!(bounds[0], bounds[1]);
            assert_eq!(bounds[0], bounds[2]);
        }
    }
    #[test]
    fn negative_entry_and_function_after_operator() {
        let mut a = app();
        a.insert("2*-4");
        a.clear_entry();
        assert_eq!(a.expression, "2*");
        a.unary("sin");
        a.insert("30)");
        a.evaluate();
        assert!((a.value.unwrap().as_f64() - 1.0).abs() < 1e-14);
    }

    #[test]
    fn editor_typing_after_equals_starts_fresh_or_continues_result() {
        for (text, expression, result) in [("4", "4", "4"), ("+4", "5+4", "9")] {
            let mut a = app();
            let ctx = egui::Context::default();
            editor_frame(&mut a, &ctx, vec![]);
            ctx.memory_mut(|m| m.request_focus(egui::Id::new("expression_editor")));
            editor_frame(
                &mut a,
                &ctx,
                vec![
                    egui::Event::Text("2+3".into()),
                    key(egui::Key::Enter, egui::Modifiers::NONE),
                ],
            );
            editor_frame(&mut a, &ctx, vec![egui::Event::Text(text.into())]);
            assert_eq!(a.expression, expression);
            assert_eq!(a.result, result);
        }
    }
    #[test]
    fn evaluated_expression_can_still_be_corrected_at_caret() {
        let mut a = app();
        let ctx = egui::Context::default();
        editor_frame(&mut a, &ctx, vec![]);
        ctx.memory_mut(|m| m.request_focus(egui::Id::new("expression_editor")));
        editor_frame(
            &mut a,
            &ctx,
            vec![
                egui::Event::Text("12+3".into()),
                key(egui::Key::Enter, egui::Modifiers::NONE),
            ],
        );
        editor_frame(
            &mut a,
            &ctx,
            vec![key(egui::Key::Home, egui::Modifiers::NONE)],
        );
        editor_frame(&mut a, &ctx, vec![egui::Event::Text("2".into())]);
        assert_eq!(a.expression, "212+3");
        assert_eq!(a.result, "215");
    }
    #[test]
    fn nth_root_retains_evaluated_base() {
        let mut a = app();
        a.insert("4+4");
        a.evaluate();
        a.nth_root();
        a.insert("3)");
        a.evaluate();
        assert_eq!(a.expression, "8^(1/3)");
        assert_eq!(a.result, "2");
        assert!(a.result_valid);
    }
    #[test]
    fn recalled_approximation_survives_continuation_and_repeat() {
        let mut a = app();
        a.insert("pi");
        a.evaluate();
        let entry = a.history.entries[0].clone();
        a.reuse(&entry);
        assert!(a.approximate && a.result_valid);
        a.insert("+");
        a.insert("1");
        a.evaluate();
        assert!(a.approximate);
        a.evaluate();
        assert!(a.approximate);
        assert!(a.history.entries.last().unwrap().approximate);
        a.insert("4");
        assert!(!a.approximate);
        a.clear();
        a.insert("2+3");
        a.evaluate();
        let entry = a.history.entries.last().unwrap().clone();
        a.reuse(&entry);
        assert!(!a.approximate);
    }
    #[test]
    fn angle_switch_recalculates_without_committing_history() {
        let mut a = app();
        a.insert("sin(30)");
        a.evaluate();
        assert_eq!(a.result, "0.5");
        a.toggle_angle();
        assert_eq!(a.settings.angle, Angle::Radians);
        assert_eq!(
            a.result,
            calculator::evaluate("sin(30)", Angle::Radians)
                .unwrap()
                .value
                .text()
        );
        assert!(!a.evaluated);
        assert_eq!(a.history.entries.len(), 1);
        a.evaluate();
        assert_eq!(a.history.entries.len(), 2);
        a.toggle_angle();
        assert_eq!(a.result, "0.5");
    }
    #[test]
    fn previews_distinguish_errors_from_unfinished_input() {
        let mut a = app();
        for expression in ["2+", "sin(", "(2+3"] {
            a.edit_expression(expression.into());
            assert!(!a.result_valid);
            assert!(a.error.is_none(), "{expression}");
        }
        for expression in ["1/0", "sqrt(-1)", "2**3", "(2+3x)"] {
            a.edit_expression(expression.into());
            assert!(!a.result_valid);
            assert!(a.error.is_some(), "{expression}");
        }
        a.edit_expression("1/2".into());
        assert_eq!(a.result, "0.5");
        assert!(a.error.is_none() && a.result_valid);
    }
    #[test]
    fn escape_dismisses_floating_panels_before_clearing_input() {
        let mut a = app();
        a.settings.history_open = true;
        a.help_open = true;
        a.insert("123");
        let ctx = egui::Context::default();
        let escape = || vec![key(egui::Key::Escape, egui::Modifiers::NONE)];
        editor_frame(&mut a, &ctx, escape());
        assert!(!a.help_open && a.settings.history_open);
        assert_eq!(a.expression, "123");
        editor_frame(&mut a, &ctx, escape());
        assert!(!a.settings.history_open);
        assert_eq!(a.expression, "123");
        editor_frame(&mut a, &ctx, escape());
        assert!(a.expression.is_empty());
    }

    #[test]
    fn display_aligns_editable_expression_with_result_and_keeps_long_results_single_line() {
        for scientific in [false, true] {
            let mut a = app();
            a.settings.scientific = scientific;
            let ctx = egui::Context::default();
            crate::ui::install_fonts(&ctx);
            a.edit_expression("128*0.85".into());
            editor_frame(&mut a, &ctx, vec![]);
            let output = editor_frame(&mut a, &ctx, vec![]);
            let bounds = |value: &str| {
                output
                    .shapes
                    .iter()
                    .find_map(|shape| match &shape.shape {
                        egui::epaint::Shape::Text(text) if text.galley.text() == value => {
                            Some(text.visual_bounding_rect())
                        }
                        _ => None,
                    })
                    .unwrap_or_else(|| panic!("Missing {value}"))
            };
            let expression = bounds("128*0.85");
            let result = bounds("108.8");
            assert!(
                (expression.right() - result.right()).abs() < 4.0,
                "{expression:?}, {result:?}"
            );
            let gap = result.top() - expression.bottom();
            assert!((0.0..24.0).contains(&gap), "Expression/result gap: {gap}");
            a.edit_expression("1234567890123456789012345678+0".into());
            let output = editor_frame(&mut a, &ctx, vec![]);
            let result = output
                .shapes
                .iter()
                .find_map(|shape| match &shape.shape {
                    egui::epaint::Shape::Text(text)
                        if text.galley.text() == "1234567890123456789012345678" =>
                    {
                        Some(text)
                    }
                    _ => None,
                })
                .expect("long result remains rendered");
            assert_eq!(
                result.galley.rows.len(),
                1,
                "Long results must scroll, never wrap"
            );
        }
    }

    #[test]
    fn statuses_and_scientific_controls_fit_minimum_window() {
        for theme in [crate::settings::Theme::Light, crate::settings::Theme::Dark] {
            for scientific in [false, true] {
                for expression in ["", "2+", "1/0", "pi", "1234567890123456789012345678"] {
                    let mut a = app();
                    a.settings.theme = theme;
                    a.settings.scientific = scientific;
                    a.edit_expression(expression.into());
                    let ctx = egui::Context::default();
                    crate::ui::install_fonts(&ctx);
                    editor_frame(&mut a, &ctx, vec![]);
                    let output = editor_frame(&mut a, &ctx, vec![]);
                    let labels: &[&str] = if scientific {
                        &[
                            "=", "sin", "cos", "tan", "asin", "acos", "atan", "xʸ", "ʸ√x", "π",
                            "n!", "DEG", "RAD",
                        ]
                    } else {
                        &["=", "Standard"]
                    };
                    for &label in labels {
                        let shape = output
                            .shapes
                            .iter()
                            .find_map(|shape| match &shape.shape {
                                egui::epaint::Shape::Text(text) if text.galley.text() == label => {
                                    Some((text.visual_bounding_rect(), shape.clip_rect))
                                }
                                _ => None,
                            })
                            .unwrap_or_else(|| panic!("Missing {label}"));
                        assert!(
                            shape.1.contains_rect(shape.0),
                            "{label} clipped for {expression}, scientific={scientific}: {shape:?}"
                        );
                    }
                    if scientific {
                        let sin = output
                            .shapes
                            .iter()
                            .find_map(|shape| match &shape.shape {
                                egui::epaint::Shape::Text(text) if text.galley.text() == "sin" => {
                                    Some(text.visual_bounding_rect())
                                }
                                _ => None,
                            })
                            .unwrap();
                        assert!(output.shapes.iter().any(|shape| matches!(&shape.shape,
                            egui::epaint::Shape::Rect(key) if key.rect.contains(sin.center()) && key.rect.height() >= 32.0)));
                    }
                }
            }
        }
    }
}
