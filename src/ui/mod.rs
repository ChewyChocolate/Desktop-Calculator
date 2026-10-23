use crate::{app::CalculatorApp, calculator::Angle, settings::Theme};
use eframe::egui::{self, Align, Color32, FontId, Layout, RichText, Stroke, Vec2};

#[derive(Clone, Copy)]
struct Palette {
    bg: Color32,
    panel: Color32,
    key: Color32,
    special: Color32,
    science: Color32,
    ink: Color32,
    muted: Color32,
    line: Color32,
    accent: Color32,
    accent_soft: Color32,
    error: Color32,
}
// Logical-point tokens keep the same rhythm at every DPI and window size.
const GAP: f32 = 8.0;
const CONTENT_WIDTH: f32 = 480.0;
const RADIUS: u8 = 6;
const TOOL_SIZE: f32 = 32.0;
pub(crate) const HISTORY_BREAKPOINT: f32 = 720.0;

fn palette(ctx: &egui::Context) -> Palette {
    if ctx.style().visuals.dark_mode {
        Palette {
            bg: hex(0x191e1c),
            panel: hex(0x151a18),
            key: hex(0x252c28),
            special: hex(0x364139),
            science: hex(0x293831),
            ink: hex(0xeaf0e9),
            muted: hex(0xa4b1a7),
            line: hex(0x38433c),
            accent: hex(0x9bd1ae),
            accent_soft: hex(0x304e3d),
            error: hex(0xffb5a2),
        }
    } else {
        Palette {
            bg: hex(0xf5f5ef),
            panel: hex(0xeceee6),
            key: hex(0xffffff),
            special: hex(0xe6eae1),
            science: hex(0xe5ede5),
            ink: hex(0x243c30),
            muted: hex(0x647468),
            line: hex(0xd6ddd1),
            accent: hex(0x245d42),
            accent_soft: hex(0xdce8db),
            error: hex(0x9c332b),
        }
    }
}
fn hex(h: u32) -> Color32 {
    Color32::from_rgb((h >> 16) as u8, (h >> 8) as u8, h as u8)
}
pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "math-fallback".into(),
        egui::FontData::from_static(include_bytes!("../../assets/DejaVuSans.ttf")).into(),
    );
    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        fonts
            .families
            .entry(family)
            .or_default()
            .push("math-fallback".into());
    }
    ctx.set_fonts(fonts);
}
pub fn apply_theme(ctx: &egui::Context, theme: Theme) {
    ctx.set_theme(match theme {
        Theme::System => egui::ThemePreference::System,
        Theme::Light => egui::ThemePreference::Light,
        Theme::Dark => egui::ThemePreference::Dark,
    });
    let p = palette(ctx);
    ctx.style_mut(|s| {
        s.visuals.panel_fill = p.bg;
        s.visuals.window_fill = p.bg;
        s.visuals.override_text_color = Some(p.ink);
        s.visuals.selection.bg_fill = p.accent_soft;
        s.visuals.selection.stroke = Stroke::new(1.5_f32, p.accent);
        s.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, p.line);
        s.spacing.item_spacing = Vec2::splat(GAP);
        s.spacing.button_padding = Vec2::new(12.0, 8.0);
        s.visuals.widgets.inactive.corner_radius = RADIUS.into();
        s.visuals.widgets.inactive.bg_fill = p.special;
        s.visuals.widgets.inactive.weak_bg_fill = p.special;
        s.visuals.widgets.inactive.bg_stroke = Stroke::NONE;
        s.visuals.widgets.hovered.bg_fill = p.accent_soft;
        s.visuals.widgets.hovered.weak_bg_fill = p.accent_soft;
        s.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, p.line);
        s.visuals.widgets.active.bg_fill = p.accent_soft;
        s.visuals.widgets.active.weak_bg_fill = p.accent_soft;
        s.visuals.widgets.active.bg_stroke = Stroke::new(1.5_f32, p.accent);
        s.visuals.widgets.hovered.corner_radius = RADIUS.into();
        s.visuals.widgets.active.corner_radius = RADIUS.into();
        s.animation_time = 0.0;
    });
}
fn icon_button(ui: &mut egui::Ui, label: &str, tip: &str, selected: bool) -> egui::Response {
    let response = ui
        .scope(|ui| {
            ui.spacing_mut().button_padding = Vec2::ZERO;
            ui.add_sized(
                [TOOL_SIZE, TOOL_SIZE],
                egui::Button::new(RichText::new(""))
                    .selected(selected)
                    .frame_when_inactive(label != "⧉"),
            )
        })
        .inner;
    paint_icon(ui, &response, label);
    focus_ring(ui, &response, palette(ui.ctx()));
    response
        .widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), tip));
    response.on_hover_text(tip)
}
// All toolbar and editing icons share an 18-point canvas and 1.5-point stroke.
fn paint_icon(ui: &egui::Ui, response: &egui::Response, icon: &str) {
    let c = response.rect.center();
    let stroke = Stroke::new(1.5_f32, ui.style().interact(response).fg_stroke.color);
    let painter = ui
        .painter()
        .with_clip_rect(response.rect.intersect(ui.clip_rect()));
    let line = |a: [f32; 2], b: [f32; 2]| {
        painter.line_segment([c + Vec2::from(a), c + Vec2::from(b)], stroke);
    };
    match icon {
        "⧉" => {
            for offset in [Vec2::new(-2.0, -2.0), Vec2::new(2.0, 2.0)] {
                painter.rect_stroke(
                    egui::Rect::from_center_size(c + offset, Vec2::new(9.0, 11.0)),
                    1,
                    stroke,
                    egui::StrokeKind::Inside,
                );
            }
        }
        "◷" => {
            painter.circle_stroke(c, 7.0, stroke);
            line([0.0, -4.0], [0.0, 0.0]);
            line([0.0, 0.0], [4.0, 0.0]);
        }
        "×" => {
            line([-5.0, -5.0], [5.0, 5.0]);
            line([-5.0, 5.0], [5.0, -5.0]);
        }
        "◐" => {
            painter.circle_stroke(c, 7.0, stroke);
            painter.add(egui::Shape::convex_polygon(
                (0..=16)
                    .map(|i| {
                        let angle =
                            -std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::PI / 16.0;
                        c + Vec2::angled(angle) * 5.0
                    })
                    .collect(),
                stroke.color,
                Stroke::NONE,
            ));
        }
        "⌫" => {
            let points = [
                [-9.0, 0.0],
                [-4.0, -6.0],
                [9.0, -6.0],
                [9.0, 6.0],
                [-4.0, 6.0],
                [-9.0, 0.0],
            ];
            painter.add(egui::Shape::line(
                points.map(|p| c + Vec2::from(p)).to_vec(),
                stroke,
            ));
            line([-1.0, -3.0], [5.0, 3.0]);
            line([-1.0, 3.0], [5.0, -3.0]);
        }
        _ => {}
    }
}

fn focus_ring(ui: &egui::Ui, response: &egui::Response, p: Palette) {
    if response.has_focus() {
        ui.painter().rect_stroke(
            response.rect.shrink(3.0),
            RADIUS,
            Stroke::new(2.0_f32, p.accent),
            egui::StrokeKind::Inside,
        );
    }
}
pub fn show(app: &mut CalculatorApp, ctx: &egui::Context) {
    let p = palette(ctx);
    // The rail only docks when at least 440 points remain for the calculator.
    let wide = ctx.content_rect().width() >= HISTORY_BREAKPOINT;
    if app.settings.history_open && wide {
        egui::SidePanel::right("history")
            .default_width(232.0)
            .width_range(208.0..=(ctx.content_rect().width() - 440.0).min(320.0))
            .resizable(true)
            .frame(egui::Frame::new().fill(p.panel).inner_margin(16))
            .show(ctx, |ui| history(app, ui, p));
    }
    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(p.bg).inner_margin(16))
        .show(ctx, |ui| {
            let width = ui.available_width().min(CONTENT_WIDTH);
            let height = ui.available_height().min(780.0);
            let rect =
                egui::Rect::from_center_size(ui.max_rect().center(), Vec2::new(width, height));
            ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                calculator_panel(app, ui, ctx, p);
            });
        });
    if app.settings.history_open && !wide {
        let mut open = true;
        egui::Window::new("Calculation history")
            .open(&mut open)
            .default_width(280.0)
            .max_width((ctx.content_rect().width() - 48.0).max(200.0))
            .max_height((ctx.content_rect().height() - 64.0).max(200.0))
            .default_height(400.0)
            .collapsible(false)
            .show(ctx, |ui| history(app, ui, p));
        if !open {
            app.settings.history_open = false;
        }
    }
    if app.help_open {
        egui::Window::new("Keyboard shortcuts")
            .open(&mut app.help_open)
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.label("Type directly, or click the expression to edit anywhere.");
                ui.label("In the editor, use standard selection, copy, paste and undo shortcuts.");
                ui.add_space(12.0);
                egui::Grid::new("shortcuts")
                    .spacing([30.0, 10.0])
                    .show(ui, |ui| {
                        let command = if cfg!(target_os = "macos") {
                            "⌘"
                        } else {
                            "Ctrl"
                        };
                        for (keys, action) in [
                            ("0–9  .  + − * /", "Type a calculation"),
                            ("( )  ^  %  !", "Group, power, percent, factorial"),
                            ("Enter / =", "Calculate; press again to repeat"),
                            ("Backspace", "Remove last character"),
                            ("Delete", "Clear last operand"),
                            ("Escape", "Clear calculation / close help"),
                            (
                                &format!("{command}+C / {command}+V"),
                                "Copy result / paste an expression",
                            ),
                            ("Ctrl+H", "Toggle history"),
                            ("Ctrl+1 / Ctrl+2", "Standard / Scientific mode"),
                            ("Tab / Shift+Tab", "Move focus between controls"),
                            ("Space", "Activate focused control"),
                            ("F1", "Show keyboard shortcuts"),
                        ] {
                            ui.label(RichText::new(keys).monospace().color(p.accent));
                            ui.label(action);
                            ui.end_row();
                        }
                    });
                ui.add_space(12.0);
                ui.label("Percent divides the preceding operand by 100.");
                ui.label("Scientific results are approximate (IEEE-754).");
                ui.label("Calculator 1.0");
            });
    }
}
fn calculator_panel(app: &mut CalculatorApp, ui: &mut egui::Ui, ctx: &egui::Context, p: Palette) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        egui::Frame::new()
            .fill(p.panel)
            .corner_radius(RADIUS)
            .inner_margin(3)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    for (label, scientific) in [("Standard", false), ("Scientific", true)] {
                        let active = app.settings.scientific == scientific;
                        if ui
                            .add_sized(
                                [88.0, TOOL_SIZE],
                                egui::Button::new(
                                    RichText::new(label).size(14.0).color(if active {
                                        p.accent
                                    } else {
                                        p.muted
                                    }),
                                )
                                .selected(active)
                                .fill(if active {
                                    p.accent_soft
                                } else {
                                    p.panel
                                }),
                            )
                            .on_hover_text(if scientific {
                                "Scientific mode · Ctrl+2"
                            } else {
                                "Standard mode · Ctrl+1"
                            })
                            .clicked()
                        {
                            app.settings.scientific = scientific;
                        }
                    }
                });
            });
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = GAP;
            if icon_button(
                ui,
                "◷",
                "Toggle history · Ctrl+H",
                app.settings.history_open,
            )
            .clicked()
            {
                app.settings.history_open = !app.settings.history_open;
            }
            let menu = ui
                .scope(|ui| {
                    ui.spacing_mut().button_padding = Vec2::splat(4.0);
                    ui.spacing_mut().interact_size = Vec2::splat(TOOL_SIZE);
                    egui::containers::menu::MenuButton::from_button(
                        egui::Button::new("").min_size(Vec2::splat(TOOL_SIZE)),
                    )
                    .ui(ui, |ui| {
                        ui.label(RichText::new("Appearance").size(13.0).color(p.muted));
                        ui.selectable_value(&mut app.settings.theme, Theme::System, "System");
                        ui.selectable_value(&mut app.settings.theme, Theme::Light, "Light");
                        ui.selectable_value(&mut app.settings.theme, Theme::Dark, "Dark");
                        ui.separator();
                        if ui.button("Keyboard shortcuts    F1").clicked() {
                            app.help_open = true;
                            ui.close();
                        }
                    })
                    .0
                })
                .inner;
            paint_icon(ui, &menu, "◐");
            menu.widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::WidgetType::Button,
                    true,
                    "Appearance and keyboard shortcuts",
                )
            });
            menu.on_hover_text("Appearance and keyboard shortcuts");
        });
    });
    ui.add_space(GAP);
    display(app, ui, ctx, p);
    ui.add_space(GAP);
    let sci_height = if app.settings.scientific { 178.0 } else { 0.0 };
    // Whole-point rows leave rounding room so a fractional overflow cannot summon a scrollbar.
    let key_height = ((ui.available_height() - sci_height - GAP * 5.0 - 1.0) / 6.0)
        .floor()
        .clamp(38.0, 72.0);
    egui::ScrollArea::vertical()
        .id_salt("keypad_scroll")
        .show(ui, |ui| {
            if app.settings.scientific {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    let degrees = app.settings.angle == Angle::Degrees;
                    for (label, selected) in [("DEG", degrees), ("RAD", !degrees)] {
                        if ui
                            .add_sized(
                                [48.0, 28.0],
                                egui::Button::new(RichText::new(label).size(13.0))
                                    .selected(selected),
                            )
                            .on_hover_text(format!(
                                "Angle unit: {}",
                                if label == "DEG" { "degrees" } else { "radians" }
                            ))
                            .clicked()
                            && !selected
                        {
                            app.toggle_angle();
                        }
                    }
                });
                let science = [
                    [
                        ("sin", "sin"),
                        ("cos", "cos"),
                        ("tan", "tan"),
                        ("xʸ", "^"),
                        ("ʸ√x", "root"),
                    ],
                    [
                        ("asin", "asin"),
                        ("acos", "acos"),
                        ("atan", "atan"),
                        ("ln", "ln"),
                        ("log", "log"),
                    ],
                    [("π", "pi"), ("e", "e"), ("n!", "!"), ("(", "("), (")", ")")],
                ];
                for row in science {
                    key_row(app, ui, &row, 34.0, p, true);
                }
                ui.add_space(6.0);
            }
            let rows = [
                [("CE", "ce"), ("C", "clear"), ("⌫", "back"), ("÷", "/")],
                [
                    ("1/x", "reciprocal"),
                    ("x²", "square"),
                    ("√x", "sqrt"),
                    ("×", "*"),
                ],
                [("7", "7"), ("8", "8"), ("9", "9"), ("−", "-")],
                [("4", "4"), ("5", "5"), ("6", "6"), ("+", "+")],
                [("1", "1"), ("2", "2"), ("3", "3"), ("%", "%")],
                [("±", "sign"), ("0", "0"), (".", "."), ("=", "=")],
            ];
            for row in rows {
                key_row(app, ui, &row, key_height, p, false);
            }
        });
}
fn display(app: &mut CalculatorApp, ui: &mut egui::Ui, ctx: &egui::Context, p: Palette) {
    let result_height = 50.0;
    egui::Frame::new()
        .fill(p.panel)
        .corner_radius(RADIUS)
        .inner_margin(if app.settings.scientific { 8 } else { 12 })
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 2.0;
            let mut expression = app.expression.clone();
            let editor = ui.add_sized(
                [(ui.available_width() - TOOL_SIZE - GAP).max(1.0), 24.0],
                egui::TextEdit::singleline(&mut expression)
                    .id(egui::Id::new("expression_editor"))
                    .hint_text("Type a calculation…")
                    .margin(Vec2::ZERO)
                    .font(FontId::proportional(17.0))
                    .text_color(p.muted)
                    .frame(false)
                    .horizontal_align(Align::RIGHT)
                    .char_limit(2048)
                    .desired_width(f32::INFINITY),
            );
            editor.clone().on_hover_text(
                "Edit anywhere · standard selection, copy, paste and undo shortcuts",
            );
            if editor.clicked() {
                app.begin_edit();
            }
            if editor.changed() {
                app.edit_expression(expression);
            }
            ui.horizontal(|ui| {
                let result = if app.result_valid && app.approximate {
                    format!("≈ {}", app.result)
                } else {
                    app.result.clone()
                };
                let width = (ui.available_width() - TOOL_SIZE - GAP).max(1.0);
                let size = ui.fonts_mut(|fonts| {
                    let measured = fonts
                        .layout_no_wrap(result.clone(), FontId::monospace(44.0), p.ink)
                        .size()
                        .x;
                    (44.0 * width / measured.max(1.0)).clamp(20.0, 44.0)
                });
                ui.allocate_ui_with_layout(
                    Vec2::new(width, result_height),
                    Layout::right_to_left(Align::Center),
                    |ui| {
                        egui::ScrollArea::horizontal()
                            .id_salt("result_scroll")
                            .max_height(result_height)
                            .stick_to_right(true)
                            .show(ui, |ui| {
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(&result).font(FontId::monospace(size)).color(
                                            if app.result_valid || app.expression.is_empty() {
                                                p.ink
                                            } else {
                                                p.muted
                                            },
                                        ),
                                    )
                                    .wrap_mode(egui::TextWrapMode::Extend)
                                    .selectable(false),
                                );
                            });
                    },
                );
                if ui
                    .add_enabled_ui(app.result_valid, |ui| {
                        icon_button(ui, "⧉", "Copy result · Ctrl/Cmd+C", false)
                    })
                    .inner
                    .clicked()
                {
                    app.copy(ctx);
                }
            });
            let status = if let Some(error) = &app.error {
                (error.as_str(), p.error)
            } else if let Some((notice, _)) = &app.notice {
                (notice.as_str(), p.accent)
            } else if app.expression.is_empty() {
                ("", p.muted)
            } else if !app.result_valid {
                ("Incomplete expression · previous result", p.muted)
            } else if app.approximate {
                ("Approximate result", p.muted)
            } else if app.evaluated {
                ("Result", p.muted)
            } else {
                ("Live preview", p.muted)
            };
            // Reserve one line so errors and copy feedback never move the keypad.
            let (rect, _) =
                ui.allocate_exact_size(Vec2::new(ui.available_width(), 20.0), egui::Sense::hover());
            ui.put(
                rect,
                egui::Label::new(RichText::new(status.0).size(13.0).color(status.1)).truncate(),
            )
            .on_hover_text(status.0);
        });
}
fn key_row<const N: usize>(
    app: &mut CalculatorApp,
    ui: &mut egui::Ui,
    keys: &[(&str, &str); N],
    height: f32,
    p: Palette,
    science: bool,
) {
    let width = (ui.available_width() - GAP * (N - 1) as f32) / N as f32;
    ui.horizontal(|ui| {
        ui.spacing_mut().button_padding = Vec2::new(8.0, 4.0);
        for &(label, action) in keys {
            let operator = matches!(action, "/" | "*" | "-" | "+" | "%" | "^");
            let special = !action.chars().all(|c| c.is_ascii_digit() || c == '.');
            let fill = if action == "=" {
                p.accent
            } else if operator {
                p.accent_soft
            } else if science {
                p.science
            } else if special {
                p.special
            } else {
                p.key
            };
            let ink = if action == "=" {
                if ui.visuals().dark_mode {
                    p.panel
                } else {
                    Color32::WHITE
                }
            } else if operator {
                p.accent
            } else {
                p.ink
            };
            let response = ui
                .scope(|ui| {
                    let dark = ui.visuals().dark_mode;
                    let widgets = &mut ui.style_mut().visuals.widgets;
                    widgets.inactive.weak_bg_fill = fill;
                    widgets.inactive.bg_fill = fill;
                    widgets.inactive.bg_stroke = Stroke::new(0.5_f32, p.line);
                    widgets.hovered.weak_bg_fill =
                        fill.gamma_multiply(if dark { 1.12 } else { 0.96 });
                    widgets.hovered.bg_fill = widgets.hovered.weak_bg_fill;
                    widgets.hovered.bg_stroke = Stroke::new(1.0_f32, p.line);
                    widgets.active.weak_bg_fill = fill.gamma_multiply(0.88);
                    widgets.active.bg_fill = widgets.active.weak_bg_fill;
                    widgets.active.bg_stroke = Stroke::new(1.5_f32, p.accent);
                    ui.add_sized(
                        [width, height],
                        egui::Button::new(
                            RichText::new(if action == "back" { "" } else { label })
                                .size(if science {
                                    if matches!(action, "^" | "root" | "!" | "pi") {
                                        20.0
                                    } else {
                                        17.0
                                    }
                                } else {
                                    22.0
                                })
                                .color(ink),
                        )
                        .corner_radius(RADIUS),
                    )
                })
                .inner;
            if action == "back" {
                paint_icon(ui, &response, "⌫");
            }
            focus_ring(ui, &response, p);
            // A pending binary operator has an underline, independent of keyboard focus.
            if matches!(action, "/" | "*" | "-" | "+" | "^")
                && app.expression.ends_with(action)
                && !app.evaluated
            {
                let r = response.rect.shrink(7.0);
                ui.painter().line_segment(
                    [r.left_bottom(), r.right_bottom()],
                    Stroke::new(2.0_f32, p.accent),
                );
            }
            let tip = match action {
                "ce" => "Clear entry · Delete",
                "clear" => "Clear all · Escape",
                "back" => "Backspace",
                "reciprocal" => "Reciprocal of last operand",
                "square" => "Square last operand",
                "sqrt" => "Square root of last operand",
                "sign" => "Change sign of last operand",
                "%" => "Percent: divide operand by 100",
                "=" => "Calculate · Enter (again to repeat)",
                "root" => "Nth root: x^(1/n). Enter the root degree, then close )",
                "^" => "Raise to a power",
                "!" => "Factorial (whole numbers, 0–170)",
                "pi" => "Pi · approximate",
                "e" => "Euler's number · approximate",
                "sin" => "Sine of last operand · uses DEG/RAD",
                "cos" => "Cosine of last operand · uses DEG/RAD",
                "tan" => "Tangent of last operand · uses DEG/RAD",
                "asin" => "Inverse sine · result uses DEG/RAD",
                "acos" => "Inverse cosine · result uses DEG/RAD",
                "atan" => "Inverse tangent · result uses DEG/RAD",
                "ln" => "Natural logarithm",
                "log" => "Base-10 logarithm",
                _ => "",
            };
            response.widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::WidgetType::Button,
                    ui.is_enabled(),
                    if tip.is_empty() { label } else { tip },
                )
            });
            let response = if tip.is_empty() {
                response
            } else {
                response.on_hover_text(tip)
            };
            if response.clicked() {
                match action {
                    "ce" => app.clear_entry(),
                    "clear" => app.clear(),
                    "back" => app.backspace(),
                    "=" => app.evaluate(),
                    "reciprocal" | "square" | "sqrt" | "sign" | "sin" | "cos" | "tan" | "asin"
                    | "acos" | "atan" | "ln" | "log" => app.unary(action),
                    "root" => app.nth_root(),
                    _ => app.insert(action),
                }
            }
        }
    });
}
fn display_expression(s: &str) -> String {
    s.replace('*', " × ").replace('/', " ÷ ").replace("pi", "π")
}
fn history(app: &mut CalculatorApp, ui: &mut egui::Ui, p: Palette) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("History").size(17.0).strong());
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if icon_button(ui, "×", "Close history", false).clicked() {
                app.settings.history_open = false;
            }
            ui.add_enabled_ui(!app.history.entries.is_empty(), |ui| {
                if ui
                    .small_button("Clear")
                    .on_hover_text("Clear saved calculations · Undo is available")
                    .clicked()
                {
                    app.clear_history();
                }
            });
        });
    });
    ui.label(
        RichText::new(format!("{} saved locally", app.history.entries.len()))
            .size(13.0)
            .color(p.muted),
    );
    if app.cleared_history.is_some() {
        ui.horizontal(|ui| {
            ui.label(RichText::new("History cleared").size(13.0).color(p.muted));
            if ui.button("Undo").clicked() {
                app.restore_history();
            }
        });
    }
    ui.add_space(GAP);
    if app.history.entries.is_empty() {
        ui.add_space(24.0);
        ui.label(RichText::new("No calculations yet").size(16.0).color(p.ink));
        ui.label(
            RichText::new(
                "Press Enter or = to save a calculation. Select an entry to restore it here.",
            )
            .size(13.0)
            .color(p.muted),
        );
    }
    let mut recall = None;
    egui::ScrollArea::vertical()
        .id_salt("history_list")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 4.0;
            for (i, entry) in app.history.entries.iter().enumerate().rev() {
                ui.push_id(i, |ui| {
                    let (row, _) = ui.allocate_exact_size(
                        Vec2::new(ui.available_width(), 52.0),
                        egui::Sense::hover(),
                    );
                    let select = egui::Rect::from_min_max(
                        row.min,
                        row.max - Vec2::new(TOOL_SIZE + GAP, 0.0),
                    );
                    let response =
                        ui.interact(select, ui.id().with("recall"), egui::Sense::click());
                    response.widget_info(|| {
                        egui::WidgetInfo::labeled(
                            egui::WidgetType::Button,
                            true,
                            format!("Restore {} = {}", entry.expression, entry.result),
                        )
                    });
                    let selected = app.evaluated
                        && app.expression == entry.expression
                        && app.result == entry.result;
                    if selected || response.hovered() || response.has_focus() {
                        ui.painter().rect_filled(select, RADIUS, p.accent_soft);
                    }
                    if response.is_pointer_button_down_on() {
                        ui.painter().rect_filled(select, RADIUS, p.special);
                    }
                    if response.has_focus() {
                        ui.painter().rect_stroke(
                            select,
                            RADIUS,
                            Stroke::new(1.5_f32, p.accent),
                            egui::StrokeKind::Inside,
                        );
                    }
                    let width = (select.width() - 16.0).max(1.0);
                    for (text, size, ink, y, family) in [
                        (
                            display_expression(&entry.expression),
                            13.0,
                            p.muted,
                            4.0,
                            egui::FontFamily::Proportional,
                        ),
                        (
                            format!(
                                "{}{}",
                                if entry.approximate { "≈ " } else { "" },
                                entry.result
                            ),
                            19.0,
                            p.ink,
                            24.0,
                            egui::FontFamily::Monospace,
                        ),
                    ] {
                        let mut job = egui::text::LayoutJob::simple(
                            text,
                            FontId::new(size, family),
                            ink,
                            width,
                        );
                        job.wrap.max_rows = 1;
                        let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
                        ui.painter()
                            .with_clip_rect(select.intersect(ui.clip_rect()))
                            .galley(
                                egui::pos2(
                                    select.right() - 8.0 - galley.size().x,
                                    select.top() + y,
                                ),
                                galley,
                                ink,
                            );
                    }
                    if response
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .on_hover_text(format!(
                            "{} = {}\nRestore calculation",
                            entry.expression, entry.result
                        ))
                        .clicked()
                    {
                        recall = Some(entry.clone());
                    }
                    let copy_rect = egui::Rect::from_center_size(
                        egui::pos2(
                            row.right() - TOOL_SIZE / 2.0,
                            row.bottom() - TOOL_SIZE / 2.0 - 2.0,
                        ),
                        Vec2::splat(TOOL_SIZE),
                    );
                    let copy = ui
                        .scope_builder(egui::UiBuilder::new().max_rect(copy_rect), |ui| {
                            icon_button(ui, "⧉", "Copy history result", false)
                        })
                        .inner;
                    if copy.clicked() {
                        ui.ctx().copy_text(entry.result.clone());
                        app.notice = Some(("Result copied".into(), std::time::Instant::now()));
                    }
                });
            }
        });
    if let Some(entry) = recall {
        app.restore_calculation(&entry);
    }
}
