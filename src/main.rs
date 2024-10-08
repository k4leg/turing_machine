// SPDX-FileCopyrightText: 2024 k4leg <pOgtq@yandex.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread::{self, JoinHandle};

use anyhow::{anyhow, Context, Result};
use eframe::egui::text::LayoutJob;
use eframe::egui::{
    self, popup_below_widget, Align, ComboBox, Grid, Layout, RichText, ScrollArea, Sides, Style,
    WidgetText,
};
use egui_extras::{Column, Size, StripBuilder, TableBuilder};
use egui_plot::{Legend, Line, Plot};
use fluent::{FluentBundle, FluentResource};

mod exhaustive_words;
mod translations;
mod turing_machine;

use self::exhaustive_words::GetExhaustiveWords;
use self::translations::AppLanguage;
use self::turing_machine::cell::{Cell, BLANK_CHAR};
use self::turing_machine::core::{Direction, MultiCommand};
use self::turing_machine::TuringMachine;

#[derive(Clone, PartialEq)]
enum Preset {
    OneTape,
    MultiTape,
}

impl From<Preset> for WidgetText {
    fn from(value: Preset) -> Self {
        let style = Style::default();
        let mut layout_job = LayoutJob::default();
        let preset = match value {
            Preset::OneTape => "1",
            Preset::MultiTape => "2",
        };
        let mut append = |x: RichText| {
            x.append_to(
                &mut layout_job,
                &style,
                egui::FontSelection::Default,
                Align::Center,
            );
        };
        append(RichText::new(preset.to_string() + " {a"));
        let n = RichText::new("n").small_raised();
        append(n.clone());
        append(RichText::new("b"));
        append(RichText::new("m").small_raised());
        append(RichText::new("c"));
        append(n);
        append(RichText::new(" | m,n≥1 & n≠m}"));
        layout_job.into()
    }
}

struct Application {
    pixels_per_point: f32,
    tm_alphabet_primary: String,
    tm_alphabet_secondary: String,
    tm_input: String,
    is_tm_running: Arc<AtomicBool>,
    is_tm_stop_requested: Arc<AtomicBool>,
    is_tm_plotting: Arc<AtomicBool>,
    is_tm_stop_plot_requested: Arc<AtomicBool>,
    tm_preset: Preset,
    num_tapes: usize,
    tm_commands: Vec<MultiCommand>,
    tm_protocol: Arc<Mutex<Vec<Vec<String>>>>,
    tm_protocol_reversed: bool,
    tm_plot_points: Arc<Mutex<Vec<[f64; 2]>>>,
    save_protocol_msg: String,
    tm_thread: Option<JoinHandle<()>>,
    tm_plot_thread: Option<JoinHandle<()>>,
    language: AppLanguage,
    messages: FluentBundle<FluentResource>,
}

impl Application {
    const COMBO_BOX_CELL_WIDTH: f32 = 25.0;

    pub fn new(pixels_per_point: f32) -> Self {
        let language = AppLanguage::default();
        Self {
            pixels_per_point,
            tm_alphabet_primary: "abc".into(),
            tm_alphabet_secondary: "01ABC".into(),
            tm_input: "".into(),
            is_tm_running: Arc::new(AtomicBool::new(false)),
            is_tm_stop_requested: Arc::new(AtomicBool::new(false)),
            is_tm_plotting: Arc::new(AtomicBool::new(false)),
            is_tm_stop_plot_requested: Arc::new(AtomicBool::new(false)),
            tm_preset: Preset::OneTape,
            num_tapes: 1,
            tm_commands: Self::preset_one_tape(),
            tm_protocol: Arc::new(Mutex::new(Vec::new())),
            tm_protocol_reversed: true,
            tm_plot_points: Arc::new(Mutex::new(Vec::new())),
            save_protocol_msg: "".into(),
            tm_thread: None,
            tm_plot_thread: None,
            messages: language.get_bundle(),
            language,
        }
    }

    fn preset_one_tape() -> Vec<MultiCommand> {
        tm_cmds![
            // Check if input is correct.
            ["q0", Cell::Blank, "qz", '0', 'N'], // 0
            ["q0", 'b', "q18", 'b', 'N'],        // 0
            ["q0", 'c', "q18", 'c', 'N'],        // 0
            ["q0", 'a', "q1", 'A', 'R'],         // ok, next
            // q1
            ["q1", Cell::Blank, "q19", Cell::Blank, 'L'], // 0, e.g. Aaa{BLANK}
            ["q1", 'a', "q1", 'a', 'R'],                  // skip a
            ["q1", 'c', "q18", 'c', 'N'],                 // 0, e.g. Aaac
            ["q1", 'B', "q2", 'B', 'N'],                  // next
            ["q1", 'b', "q2", 'b', 'N'],                  // next
            // q2
            ["q2", Cell::Blank, "q19", Cell::Blank, 'L'], // 0, e.g. AAaB{BLANK}
            ["q2", 'a', "q18", 'a', 'N'],                 // 0, e.g. AAaBa
            ["q2", 'B', "q2", 'B', 'R'],                  // skip B
            ["q2", 'b', "q3", 'B', 'L'],                  // next
            ["q2", 'c', "q8", 'c', 'L'],                  // all b's end
            // q3
            ["q3", 'B', "q3", 'B', 'L'], // skip B
            ["q3", 'a', "q4", 'a', 'N'], // next
            ["q3", 'A', "q7", 'A', 'R'], // all a's end
            // q4
            ["q4", 'a', "q4", 'a', 'L'], // skip a
            ["q4", 'A', "q5", 'A', 'R'], // next
            // q5
            ["q5", 'a', "q1", 'A', 'R'], // next
            // q7
            ["q7", Cell::Blank, "q19", Cell::Blank, 'L'], // 0, e.g. ABb
            ["q7", 'a', "q18", 'a', 'N'],                 // 0, e.g. ABabc
            ["q7", 'B', "q7", 'B', 'R'],                  // skip B
            ["q7", 'b', "q7", 'b', 'R'],                  // skip b
            ["q7", 'c', "q6", 'c', 'L'],                  // next
            // q6
            ["q6", 'A', "q6", 'A', 'L'],                  // skip A
            ["q6", 'B', "q6", 'B', 'L'],                  // skip B
            ["q6", Cell::Blank, "q18", Cell::Blank, 'R'], // next
            ["q6", 'b', "q8", 'b', 'L'],                  // next
            ["q6", 'a', "q8", 'a', 'L'],                  // next
            // q8
            ["q8", 'b', "q8", 'b', 'L'],                 // skip b
            ["q8", 'B', "q8", 'B', 'L'],                 // skip B
            ["q8", 'a', "q8", 'a', 'L'],                 // skip a
            ["q8", 'A', "q8", 'a', 'L'],                 // A=a
            ["q8", Cell::Blank, "q9", Cell::Blank, 'R'], // next
            // q9
            ["q9", 'a', "q10", 'A', 'R'], // next
            // q10
            ["q10", 'a', "q10", 'a', 'R'], // skip a
            ["q10", 'b', "q11", 'b', 'N'], // next
            ["q10", 'B', "q11", 'B', 'N'], // next
            // q11
            ["q11", 'B', "q11", 'B', 'R'], // skip B
            ["q11", 'b', "q11", 'b', 'R'], // skip b
            ["q11", 'c', "q12", 'c', 'N'], // next
            ["q11", 'C', "q12", 'C', 'N'], // next
            // q12
            ["q12", 'a', "q18", 'a', 'N'], // 0, e.g. AAABBCCac
            ["q12", Cell::Blank, "q19", Cell::Blank, 'L'], // 0, e.g. AAAABBCCC{BLANK}
            ["q12", 'C', "q12", 'C', 'R'], // skip R
            ["q12", 'c', "q13", 'C', 'L'], // next
            // q13
            ["q13", 'C', "q13", 'C', 'L'], // skip C
            ["q13", 'B', "q14", 'B', 'N'], // next
            ["q13", 'b', "q14", 'b', 'N'], // next
            // q14
            ["q14", 'B', "q14", 'B', 'L'], // skip B
            ["q14", 'b', "q14", 'b', 'L'], // skip b
            ["q14", 'A', "q15", 'A', 'R'], // all A's end
            ["q14", 'a', "q16", 'a', 'L'], // next
            // q15
            ["q15", 'b', "q15", 'b', 'R'], // skip b
            ["q15", 'B', "q15", 'B', 'R'], // skip B
            ["q15", 'c', "q17", 'c', 'N'], // next
            ["q15", 'C', "q17", 'C', 'N'], // next
            // q17
            ["q17", 'C', "q17", 'C', 'R'],                 // skip C
            ["q17", 'c', "q18", 'c', 'N'],                 // 0, an!=cn
            ["q17", 'a', "q18", 'a', 'N'],                 // 0, e.g. a's in end
            ["q17", 'b', "q18", 'b', 'N'],                 // 0, e.g. b's in end
            ["q17", Cell::Blank, "q21", Cell::Blank, 'L'], // 1
            // q16
            ["q16", 'a', "q16", 'a', 'L'], // skip a
            ["q16", 'A', "q9", 'A', 'R'],  // next
            // q18
            ["q18", 'a', "q18", 'a', 'R'],                 // skip a
            ["q18", 'b', "q18", 'b', 'R'],                 // skip b
            ["q18", 'c', "q18", 'c', 'R'],                 // skip c
            ["q18", 'A', "q18", 'A', 'R'],                 // skip A
            ["q18", 'B', "q18", 'B', 'R'],                 // skip B
            ["q18", 'C', "q18", 'C', 'R'],                 // skip C
            ["q18", Cell::Blank, "q19", Cell::Blank, 'L'], // next
            // q19
            ["q19", 'a', "q19", Cell::Blank, 'L'], // a=blank
            ["q19", 'b', "q19", Cell::Blank, 'L'], // b=blank
            ["q19", 'c', "q19", Cell::Blank, 'L'], // c=blank
            ["q19", 'A', "q19", Cell::Blank, 'L'], // A=blank
            ["q19", 'B', "q19", Cell::Blank, 'L'], // B=blank
            ["q19", 'C', "q19", Cell::Blank, 'L'], // C=blank
            ["q19", Cell::Blank, "q20", Cell::Blank, 'R'], // next
            // q20
            ["q20", Cell::Blank, "qz", '0', 'N'], // 0
            // q21
            ["q21", 'C', "q21", Cell::Blank, 'L'], // C=blank
            ["q21", 'b', "q21", Cell::Blank, 'L'], // b=blank
            ["q21", 'B', "q21", Cell::Blank, 'L'], // B=blank
            ["q21", 'A', "q21", Cell::Blank, 'L'], // A=blank
            ["q21", Cell::Blank, "q22", Cell::Blank, 'R'], // next
            // q22
            ["q22", Cell::Blank, "qz", '1', 'N'], // 1
        ]
        .into_iter()
        .map(MultiCommand::from)
        .collect()
    }

    #[rustfmt::skip]
    fn preset_multitape() -> Vec<MultiCommand> {
        tm_mcmds![
            // q0
            ["q0", [Cell::Blank, Cell::Blank], "qz", ['0', Cell::Blank], ['N', 'N']], // 0
            ["q0", ['b', Cell::Blank], "q6", ['b', Cell::Blank], ['R', 'N']], // 0
            ["q0", ['c', Cell::Blank], "q6", ['c', Cell::Blank], ['R', 'N']], // 0

            ["q0", ['a', Cell::Blank], "q1", ['a', 'X'], ['R', 'R']], // next
            // q1
            ["q1", ['a', Cell::Blank], "q1", ['a', 'X'], ['R', 'R']], // a,blank=a,X
            ["q1", ['b', Cell::Blank], "q2", ['b', Cell::Blank], ['N', 'L']], // next

            ["q1", ['c', Cell::Blank], "q6", ['c', Cell::Blank], ['R', 'N']], // 0
            ["q1", [Cell::Blank, Cell::Blank], "q7", [Cell::Blank, Cell::Blank], ['L', 'N']], // 0
            // q2
            ["q2", ['b', 'X'], "q2", ['b', 'X'], ['R', 'L']], // next
            ["q2", ['b', Cell::Blank], "q3", ['b', Cell::Blank], ['R', 'R']], // an<bn
            ["q2", ['c', 'X'], "q5", ['c', 'X'], ['N', 'N']], // an>bn

            ["q2", ['c', Cell::Blank], "q6", ['c', Cell::Blank], ['R', 'N']], // 0, an=bn
            ["q2", [Cell::Blank, 'X'], "q7", [Cell::Blank, Cell::Blank], ['L', 'N']], // 0
            ["q2", [Cell::Blank, Cell::Blank], "q7", [Cell::Blank, Cell::Blank], ['L', 'N']], // 0
            ["q2", ['a', 'X'], "q6", ['a', Cell::Blank], ['R', 'N']], // 0
            ["q2", ['a', Cell::Blank], "q6", ['a', Cell::Blank], ['R', 'N']], // 0
            // q3
            ["q3", ['b', 'X'], "q3", ['b', 'X'], ['R', 'N']], // skip b,X
            ["q3", ['c', 'X'], "q4", ['c', 'X'], ['N', 'N']], // next

            ["q3", ['a', 'X'], "q6", ['a', Cell::Blank], ['R', 'N']], // 0
            ["q3", [Cell::Blank, 'X'], "q7", [Cell::Blank, Cell::Blank], ['L', 'N']], // 0
            // q4
            ["q4", ['c', 'X'], "q4", ['c', 'X'], ['R', 'R']], // skip c,X
            ["q4", [Cell::Blank, Cell::Blank], "q9", [Cell::Blank, Cell::Blank], ['L', 'N']], // 1

            ["q4", [Cell::Blank, 'X'], "q7", [Cell::Blank, Cell::Blank], ['L', 'N']], // 0
            ["q4", ['a', 'X'], "q6", ['a', Cell::Blank], ['R', 'N']], // 0
            ["q4", ['a', Cell::Blank], "q6", ['a', Cell::Blank], ['R', 'N']], // 0
            ["q4", ['b', 'X'], "q6", ['b', Cell::Blank], ['R', 'N']], // 0
            ["q4", ['b', Cell::Blank], "q6", ['b', Cell::Blank], ['R', 'N']], // 0
            ["q4", ['c', Cell::Blank], "q6", ['c', Cell::Blank], ['R', 'N']], // 0
            // q5
            ["q5", ['c', 'X'], "q5", ['c', 'X'], ['N', 'L']], // skip c,X
            ["q5", ['c', Cell::Blank], "q4", ['c', Cell::Blank], ['N', 'R']], // next
            // q6
            ["q6", ['a', Cell::Blank], "q6", ['a', Cell::Blank], ['R', 'N']],
            ["q6", ['b', Cell::Blank], "q6", ['b', Cell::Blank], ['R', 'N']],
            ["q6", ['c', Cell::Blank], "q6", ['c', Cell::Blank], ['R', 'N']],
            ["q6", [Cell::Blank, Cell::Blank], "q7", [Cell::Blank, Cell::Blank], ['L', 'N']],
            // q7
            ["q7", ['c', Cell::Blank], "q7", [Cell::Blank, Cell::Blank], ['L', 'N']],
            ["q7", ['b', Cell::Blank], "q7", [Cell::Blank, Cell::Blank], ['L', 'N']],
            ["q7", ['a', Cell::Blank], "q7", [Cell::Blank, Cell::Blank], ['L', 'N']],
            ["q7", [Cell::Blank, Cell::Blank], "q8", [Cell::Blank, Cell::Blank], ['R', 'N']],
            // q8
            ["q8", [Cell::Blank, Cell::Blank], "qz", ['0', Cell::Blank], ['N', 'N']],
            // q9
            ["q9", ['a', Cell::Blank], "q9", [Cell::Blank, Cell::Blank], ['L', 'N']],
            ["q9", ['b', Cell::Blank], "q9", [Cell::Blank, Cell::Blank], ['L', 'N']],
            ["q9", ['c', Cell::Blank], "q9", [Cell::Blank, Cell::Blank], ['L', 'N']],
            ["q9", [Cell::Blank, Cell::Blank], "q10", [Cell::Blank, Cell::Blank], ['R', 'N']],
            // q10
            ["q10", [Cell::Blank, Cell::Blank], "qz", ['1', Cell::Blank], ['N', 'N']],
        ]
    }

    fn set_preset(&mut self) {
        (*self.tm_protocol.lock().unwrap()).clear();
        self.tm_alphabet_primary = "abc".into();
        self.tm_input.clear();
        match self.tm_preset {
            Preset::OneTape => {
                self.num_tapes = 1;
                self.tm_alphabet_secondary = "01ABC".into();
                self.tm_commands = Self::preset_one_tape();
            }
            Preset::MultiTape => {
                self.num_tapes = 2;
                self.tm_alphabet_secondary = "01X".into();
                self.tm_commands = Self::preset_multitape();
            }
        }
    }

    fn msg(&self, m: &str) -> String {
        let pattern = self.messages.get_message(m).unwrap().value().unwrap();
        self.messages
            .format_pattern(pattern, None, &mut vec![])
            .to_string()
    }

    fn next_lang(&mut self) {
        self.language = self.language.next();
        self.messages = self.language.get_bundle();
    }

    fn main_ui(&mut self, ui: &mut egui::Ui) {
        let is_tm_running = self.is_tm_running.load(Ordering::Relaxed);
        let is_tm_plotting = self.is_tm_plotting.load(Ordering::Relaxed);
        ui.horizontal(|ui| {
            egui::widgets::global_theme_preference_switch(ui);
            ui.label(self.msg("zoom"));
            if ui.button("+").clicked() {
                self.zoom(ui.ctx(), 0.5);
            }
            if ui.button("\u{2212}").clicked() {
                self.zoom(ui.ctx(), -0.5);
            }
            if ui.button(self.msg("btn-change-language")).clicked() {
                self.next_lang();
            }
        });
        Grid::new("grid_alphabet_input")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .show(ui, |ui| {
                ui.label(self.msg("alphabet-primary"));
                let widget = egui::widgets::TextEdit::singleline(&mut self.tm_alphabet_primary);
                let response = ui.add(if is_tm_running || is_tm_plotting {
                    widget.interactive(false)
                } else {
                    widget
                });
                if response.changed() {
                    let mut seen = HashSet::<char>::from_iter(self.tm_alphabet_secondary.chars());
                    self.tm_alphabet_primary = self
                        .tm_alphabet_primary
                        .chars()
                        .filter(|&ch| seen.insert(ch) && ch != BLANK_CHAR)
                        .collect();
                }
                ui.end_row();

                ui.label(self.msg("alphabet-secondary"));
                let widget = egui::widgets::TextEdit::singleline(&mut self.tm_alphabet_secondary);
                let response = ui.add(if is_tm_running || is_tm_plotting {
                    widget.interactive(false)
                } else {
                    widget
                });
                if response.changed() {
                    let mut seen = HashSet::<char>::from_iter(self.tm_alphabet_primary.chars());
                    self.tm_alphabet_secondary = self
                        .tm_alphabet_secondary
                        .chars()
                        .filter(|&ch| seen.insert(ch) && ch != BLANK_CHAR)
                        .collect();
                }
                ui.end_row();

                ui.label(self.msg("input"));
                let widget = egui::widgets::TextEdit::singleline(&mut self.tm_input);
                let response = ui.add(if is_tm_running {
                    widget.interactive(false)
                } else {
                    widget
                });
                if response.changed() {
                    self.tm_input = self
                        .tm_input
                        .chars()
                        .filter(|&ch| self.tm_alphabet_primary.contains(ch))
                        .collect();
                }
            });
        ui.horizontal(|ui| {
            ui.add_enabled_ui(!is_tm_running && !is_tm_plotting, |ui| {
                ui.vertical(|ui| {
                    if ui.button(self.msg("command-add")).clicked() {
                        self.add_command();
                    }
                    if ui.button(self.msg("command-remove")).clicked() {
                        self.remove_command();
                    }
                });
                ui.vertical(|ui| {
                    if ui.button(self.msg("tape-add")).clicked() {
                        self.add_tape();
                    }
                    if ui.button(self.msg("tape-remove")).clicked() {
                        self.remove_tape();
                    }
                });
            });
            ui.vertical(|ui| {
                if is_tm_running {
                    ui.horizontal(|ui| {
                        if ui.button(self.msg("stop")).clicked() {
                            self.request_stop_tm();
                        }
                        ui.spinner();
                    });
                } else if ui.button(self.msg("start")).clicked() {
                    self.start_tm(ui.ctx());
                }
                ui.add_enabled_ui(!is_tm_running, |ui| {
                    let button_save_protocol = ui.button(self.msg("protocol-save"));
                    let popup_save_protocol_id = egui::Id::new("popup_save_protocol_id");
                    if button_save_protocol.clicked() {
                        let res = self.save_protocol();
                        self.save_protocol_msg = match res {
                            Ok(_) => self.msg("ok-file-saved"),
                            Err(e) => format!("{e}"),
                        };
                        ui.memory_mut(|mem| mem.toggle_popup(popup_save_protocol_id));
                    }
                    popup_below_widget(
                        ui,
                        popup_save_protocol_id,
                        &button_save_protocol,
                        egui::PopupCloseBehavior::CloseOnClick,
                        |ui| {
                            ui.set_min_width(400.0);
                            ui.label(&self.save_protocol_msg);
                        },
                    );
                });
            });
            if is_tm_plotting {
                if ui.button(self.msg("plotting-stop")).clicked() {
                    self.request_stop_plot();
                }
                ui.spinner();
            } else if ui.button(self.msg("plotting-start")).clicked() {
                self.start_plot(ui.ctx());
            }
        });
        ui.add_enabled_ui(!is_tm_running && !is_tm_plotting, |ui| {
            ComboBox::from_label(self.msg("label-presets"))
                .selected_text(self.tm_preset.clone())
                .show_ui(ui, |ui| {
                    let response1 =
                        ui.selectable_value(&mut self.tm_preset, Preset::OneTape, Preset::OneTape);
                    let response2 = ui.selectable_value(
                        &mut self.tm_preset,
                        Preset::MultiTape,
                        Preset::MultiTape,
                    );
                    if response1.clicked() || response2.clicked() {
                        self.set_preset();
                    }
                });
        });
        ui.separator();
        StripBuilder::new(ui)
            .size(Size::exact(260.0))
            .size(Size::exact(150.0))
            .size(Size::remainder())
            .horizontal(|mut strip| {
                strip.cell(|ui| {
                    ui.add_enabled_ui(!is_tm_running && !is_tm_plotting, |ui| {
                        ScrollArea::horizontal().show(ui, |ui| {
                            self.table_command_ui(ui);
                        });
                    });
                });
                strip.cell(|ui| {
                    ScrollArea::horizontal().show(ui, |ui| {
                        self.table_protocol_ui(ui);
                    });
                });
                strip.cell(|ui| {
                    let line = Line::new((*self.tm_plot_points.lock().unwrap()).to_owned())
                        .name(self.msg("line-complexity"));
                    let legend = Legend::default();
                    Plot::new("plot")
                        .x_axis_label(self.msg("axis-length-of-number"))
                        .y_axis_label(self.msg("axis-max-steps"))
                        .legend(legend)
                        .y_axis_min_width(30.0)
                        .show(ui, |ui| {
                            ui.line(line);
                        });
                });
            });
    }

    fn add_command(&mut self) {
        self.tm_commands.push(
            MultiCommand::new(
                "".into(),
                vec![Cell::Blank; self.num_tapes],
                "".into(),
                vec![Cell::Blank; self.num_tapes],
                vec![Direction::None; self.num_tapes],
            )
            .unwrap(),
        );
        (*self.tm_protocol.lock().unwrap()).clear();
    }

    fn remove_command(&mut self) {
        self.tm_commands.pop();
        (*self.tm_protocol.lock().unwrap()).clear();
    }

    fn add_tape(&mut self) {
        for cmd in self.tm_commands.iter_mut() {
            cmd.add_tape();
        }
        self.num_tapes += 1;
        (*self.tm_protocol.lock().unwrap()).clear();
    }

    fn remove_tape(&mut self) {
        if self.num_tapes == 1 {
            return;
        }
        self.num_tapes -= 1;
        for cmd in self.tm_commands.iter_mut() {
            cmd.remove_tape();
        }
        (*self.tm_protocol.lock().unwrap()).clear();
    }

    fn start_tm(&mut self, ctx: &egui::Context) {
        if self.tm_commands.is_empty() {
            return;
        }
        self.is_tm_running.store(true, Ordering::Relaxed);
        (*self.tm_protocol.lock().unwrap()).clear();
        let mut start_tapes = vec![""; self.num_tapes];
        start_tapes[0] = &self.tm_input;
        let tm = TuringMachine::from_multi(&start_tapes, self.tm_commands.to_owned()).unwrap();
        let tm_protocol = Arc::clone(&self.tm_protocol);
        let is_tm_running = Arc::clone(&self.is_tm_running);
        let is_tm_stop_requested = Arc::clone(&self.is_tm_stop_requested);
        let ctx = ctx.clone();
        self.tm_thread = Some(thread::spawn(move || {
            (*tm_protocol.lock().unwrap()).push(tm.to_strings());
            for strings in tm {
                (*tm_protocol.lock().unwrap()).push(strings);
                ctx.request_repaint();
                if is_tm_stop_requested.load(Ordering::Relaxed) {
                    break;
                }
            }
            is_tm_stop_requested.store(false, Ordering::Relaxed);
            is_tm_running.store(false, Ordering::Relaxed);
        }));
    }

    fn request_stop_tm(&mut self) {
        self.is_tm_stop_requested.store(true, Ordering::Relaxed);
    }

    fn start_plot(&mut self, ctx: &egui::Context) {
        self.is_tm_plotting.store(true, Ordering::Relaxed);
        (*self.tm_plot_points.lock().unwrap()).clear();
        let tm_commands = self.tm_commands.to_owned();
        let alphabet: Vec<char> = self.tm_alphabet_primary.chars().collect();
        let is_tm_plotting = Arc::clone(&self.is_tm_plotting);
        let is_tm_stop_plot_requested = Arc::clone(&self.is_tm_stop_plot_requested);
        let tm_plot_points = Arc::clone(&self.tm_plot_points);
        let start_state = tm_commands[0].istate.to_owned();
        let num_tapes = self.num_tapes;
        let ctx = ctx.clone();
        self.tm_plot_thread = Some(thread::spawn(move || {
            let enough = || is_tm_stop_plot_requested.load(Ordering::Relaxed);
            let mut tm =
                TuringMachine::from_multi(&vec![""; num_tapes], tm_commands.to_owned()).unwrap();
            'outer: for n in 1.. {
                let mut max_steps = 0;
                for input in alphabet.get_exhaustive_words(n) {
                    let mut steps = 0;
                    let mut start_tapes = vec![""; num_tapes];
                    start_tapes[0] = &input;
                    tm.restart(&start_tapes, start_state.to_owned()).unwrap();
                    'out: loop {
                        for _ in 0..500 {
                            if tm.next().is_none() {
                                break 'out;
                            }
                            steps += 1;
                        }
                        if enough() {
                            break 'outer;
                        }
                    }
                    if enough() {
                        break 'outer;
                    }
                    max_steps = max_steps.max(steps);
                }
                (*tm_plot_points.lock().unwrap()).push([n as f64, max_steps as f64]);
                ctx.request_repaint();
                if enough() {
                    break;
                }
            }
            is_tm_stop_plot_requested.store(false, Ordering::Relaxed);
            is_tm_plotting.store(false, Ordering::Relaxed);
        }));
    }

    fn request_stop_plot(&mut self) {
        self.is_tm_stop_plot_requested
            .store(true, Ordering::Relaxed);
    }

    fn save_protocol(&self) -> Result<()> {
        if (*self.tm_protocol.lock().unwrap()).is_empty() {
            return Err(anyhow!(self.msg("err-no-protocol")));
        }
        let path = rfd::FileDialog::new()
            .set_file_name("protocol.txt")
            .save_file();
        let path = match path {
            Some(p) => p,
            None => return Err(anyhow!(self.msg("err-no-path-given"))),
        };
        let mut file = File::create(&path)
            .context(self.msg("err-failed-to-create-open") + " " + path.to_str().unwrap())?;
        let mut protocol = String::new();
        for s in &*self.tm_protocol.lock().unwrap() {
            for t in s {
                protocol.push_str(t);
                protocol.push(' ');
            }
            _ = protocol.pop();
            protocol.push('\n');
        }
        file.write(protocol.as_bytes())
            .context(self.msg("err-failed-to-write") + " " + path.to_str().unwrap())?;
        Ok(())
    }

    fn table_command_ui(&mut self, ui: &mut egui::Ui) {
        let text_style_height = ui.text_style_height(&egui::TextStyle::Button);
        let item_spacing_height = ui.spacing().item_spacing.y;
        let pad = ui.spacing().button_padding.y * 2.0;
        let interact_height = ui.spacing().interact_size.y;
        let text_height = ((text_style_height + pad).max(interact_height) + item_spacing_height)
            * self.num_tapes as f32
            - item_spacing_height;
        let available_height = ui.available_height();
        TableBuilder::new(ui)
            .striped(true)
            .cell_layout(Layout::left_to_right(Align::Center))
            .columns(Column::auto(), 5)
            .min_scrolled_height(0.0)
            .max_scroll_height(available_height)
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong(self.msg("col-state"));
                });
                header.col(|ui| {
                    ui.strong(self.msg("col-cell"));
                });
                header.col(|ui| {
                    ui.strong(self.msg("col-state"));
                });
                header.col(|ui| {
                    ui.strong(self.msg("col-cell"));
                });
                header.col(|ui| {
                    ui.strong(self.msg("col-dir"));
                });
            })
            .body(|body| {
                body.rows(text_height, self.tm_commands.len(), |mut row| {
                    let index = row.index();
                    let col_state = self.msg("col-state");
                    row.col(|ui| {
                        ui.add(
                            egui::widgets::TextEdit::singleline(
                                &mut self.tm_commands[index].istate,
                            )
                            .desired_width(40.0)
                            .hint_text(&col_state),
                        );
                    });
                    row.col(|ui| {
                        ui.vertical_centered(|ui| {
                            for i in 0..self.num_tapes {
                                let icell = self.tm_commands[index].get_mut_icell(i).unwrap();
                                ComboBox::from_id_salt(format!("icell{i}"))
                                    .selected_text(char::from(*icell).to_string())
                                    .width(Self::COMBO_BOX_CELL_WIDTH)
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(
                                            icell,
                                            Cell::Blank,
                                            BLANK_CHAR.to_string(),
                                        );
                                        for ch in self
                                            .tm_alphabet_primary
                                            .chars()
                                            .chain(self.tm_alphabet_secondary.chars())
                                        {
                                            ui.selectable_value(
                                                icell,
                                                Cell::Symbol(ch),
                                                ch.to_string(),
                                            );
                                        }
                                    });
                            }
                        });
                    });
                    row.col(|ui| {
                        ui.add(
                            egui::widgets::TextEdit::singleline(
                                &mut self.tm_commands[index].ostate,
                            )
                            .desired_width(40.0)
                            .hint_text(&col_state),
                        );
                    });
                    row.col(|ui| {
                        ui.vertical_centered(|ui| {
                            for i in 0..self.num_tapes {
                                let ocell = self.tm_commands[index].get_mut_ocell(i).unwrap();
                                ComboBox::from_id_salt(format!("ocell{i}"))
                                    .selected_text(char::from(*ocell).to_string())
                                    .width(Self::COMBO_BOX_CELL_WIDTH)
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(
                                            ocell,
                                            Cell::Blank,
                                            BLANK_CHAR.to_string(),
                                        );
                                        for ch in self
                                            .tm_alphabet_primary
                                            .chars()
                                            .chain(self.tm_alphabet_secondary.chars())
                                        {
                                            ui.selectable_value(
                                                ocell,
                                                Cell::Symbol(ch),
                                                ch.to_string(),
                                            );
                                        }
                                    });
                            }
                        });
                    });
                    row.col(|ui| {
                        ui.vertical_centered(|ui| {
                            for i in 0..self.num_tapes {
                                let direction =
                                    self.tm_commands[index].get_mut_direction(i).unwrap();
                                ComboBox::from_id_salt(format!("direction{i}"))
                                    .selected_text(direction.to_string())
                                    .width(Self::COMBO_BOX_CELL_WIDTH)
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(
                                            direction,
                                            Direction::Left,
                                            Direction::Left.to_string(),
                                        );
                                        ui.selectable_value(
                                            direction,
                                            Direction::None,
                                            Direction::None.to_string(),
                                        );
                                        ui.selectable_value(
                                            direction,
                                            Direction::Right,
                                            Direction::Right.to_string(),
                                        );
                                    });
                            }
                        });
                    });
                });
            });
    }

    fn table_protocol_ui(&mut self, ui: &mut egui::Ui) {
        let text_height = ui.text_style_height(&egui::TextStyle::Body) * self.num_tapes as f32;
        let available_height = ui.available_height();
        let length = (*self.tm_protocol.lock().unwrap()).len();
        TableBuilder::new(ui)
            .striped(true)
            .cell_layout(Layout::left_to_right(Align::Center))
            .column(Column::auto())
            .column(Column::remainder())
            .min_scrolled_height(0.0)
            .max_scroll_height(available_height)
            .header(20.0, |mut header| {
                header.col(|ui| {
                    let number_sign = self.msg("label-number-sign");
                    Sides::new().show(
                        ui,
                        |ui| {
                            ui.strong(number_sign);
                        },
                        |ui| {
                            self.tm_protocol_reversed ^= ui
                                .button(
                                    RichText::new(if self.tm_protocol_reversed {
                                        "+"
                                    } else {
                                        "\u{2212}"
                                    })
                                    .strong(),
                                )
                                .clicked();
                        },
                    );
                });
                header.col(|ui| {
                    ui.strong(self.msg("col-protocol"));
                });
            })
            .body(|body| {
                body.rows(text_height, length, |mut row| {
                    let index = if self.tm_protocol_reversed {
                        length - 1 - row.index()
                    } else {
                        row.index()
                    };
                    row.col(|ui| {
                        ui.label(index.to_string());
                    });
                    row.col(|ui| {
                        ui.label((*self.tm_protocol.lock().unwrap())[index].join("\n"));
                    });
                });
            });
    }

    fn zoom(&mut self, ctx: &egui::Context, inc: f32) {
        let zoom = self.pixels_per_point + inc;
        if (1.0..=5.0).contains(&zoom) {
            self.pixels_per_point = zoom;
            ctx.set_zoom_factor(self.pixels_per_point);
        }
    }

    fn join_threads(&mut self) {
        if !self.is_tm_running.load(Ordering::Relaxed) {
            if let Some(jh) = self.tm_thread.take() {
                _ = jh.join();
            }
        }
        if !self.is_tm_plotting.load(Ordering::Relaxed) {
            if let Some(jh) = self.tm_plot_thread.take() {
                _ = jh.join();
            }
        }
    }
}

impl eframe::App for Application {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.join_threads();
        egui::CentralPanel::default().show(ctx, |ui| self.main_ui(ui));
        if ctx.input(|i| i.viewport().close_requested()) {
            if self.is_tm_running.load(Ordering::Relaxed) {
                self.request_stop_tm();
                if let Some(jh) = self.tm_thread.take() {
                    _ = jh.join();
                }
            }
            if self.is_tm_plotting.load(Ordering::Relaxed) {
                self.request_stop_plot();
                if let Some(jh) = self.tm_plot_thread.take() {
                    _ = jh.join();
                }
            }
        }
    }
}

fn main() -> eframe::Result {
    eframe::run_native(
        "Turing Machine",
        eframe::NativeOptions::default(),
        Box::new(|c| {
            Ok(Box::new(Application::new(
                c.egui_ctx.native_pixels_per_point().unwrap_or(1.0),
            )))
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::turing_machine::cell::BLANK_CHAR;

    #[test]
    fn test_one_tape_1() {
        let mut tm = TuringMachine::from_multi(&[""], Application::preset_one_tape()).unwrap();
        assert_eq!(tm.to_strings(), [format!("q0{BLANK_CHAR}")]);
        assert_eq!(tm.next().unwrap(), ["qz0"]);
        assert_eq!(tm.next(), None);
    }

    #[test]
    fn test_one_tape_2() {
        let alphabet = &['a', 'b', 'c'];
        let preset = Application::preset_one_tape();
        for word in alphabet
            .get_exhaustive_words(3)
            .chain(alphabet.get_exhaustive_words(2))
            .chain(alphabet.get_exhaustive_words(1))
        {
            let tm = TuringMachine::from_multi(&[&word], preset.clone()).unwrap();
            assert_eq!(tm.last().unwrap(), ["qz0"]);
        }
    }

    #[test]
    fn test_one_tape_3() {
        let tm = TuringMachine::from_multi(&["aabcc"], Application::preset_one_tape()).unwrap();
        assert_eq!(tm.last().unwrap(), ["qz1"]);
    }

    #[test]
    fn test_one_tape_4() {
        let tm = TuringMachine::from_multi(&["abbc"], Application::preset_one_tape()).unwrap();
        assert_eq!(tm.last().unwrap(), ["qz1"]);
    }

    #[test]
    fn test_one_tape_5() {
        let tm = TuringMachine::from_multi(&["ababc"], Application::preset_one_tape()).unwrap();
        assert_eq!(tm.last().unwrap(), ["qz0"]);
    }

    #[test]
    fn test_one_tape_6() {
        let tm = TuringMachine::from_multi(&["abcbc"], Application::preset_one_tape()).unwrap();
        assert_eq!(tm.last().unwrap(), ["qz0"]);
    }

    #[test]
    fn test_one_tape_7() {
        let tm = TuringMachine::from_multi(&["aaabbccc"], Application::preset_one_tape()).unwrap();
        assert_eq!(tm.last().unwrap(), ["qz1"]);
    }

    #[test]
    fn test_one_tape_8() {
        let tm = TuringMachine::from_multi(&["aaabbccac"], Application::preset_one_tape()).unwrap();
        assert_eq!(tm.last().unwrap(), ["qz0"]);
    }

    #[test]
    fn test_one_tape_9() {
        let tm = TuringMachine::from_multi(&["aaabbccca"], Application::preset_one_tape()).unwrap();
        assert_eq!(tm.last().unwrap(), ["qz0"]);
    }

    #[test]
    fn test_one_tape_10() {
        let tm = TuringMachine::from_multi(&["aaabbcccb"], Application::preset_one_tape()).unwrap();
        assert_eq!(tm.last().unwrap(), ["qz0"]);
    }

    #[test]
    fn test_one_tape_11() {
        let tm = TuringMachine::from_multi(&["aaabbcccc"], Application::preset_one_tape()).unwrap();
        assert_eq!(tm.last().unwrap(), ["qz0"]);
    }

    #[test]
    fn test_one_tape_12() {
        let tm = TuringMachine::from_multi(&["aaaabbccc"], Application::preset_one_tape()).unwrap();
        assert_eq!(tm.last().unwrap(), ["qz0"]);
    }

    #[test]
    fn test_one_tape_13() {
        let tm =
            TuringMachine::from_multi(&["aaaabbbbccc"], Application::preset_one_tape()).unwrap();
        assert_eq!(tm.last().unwrap(), ["qz0"]);
    }

    #[test]
    fn test_one_tape_14() {
        let tm =
            TuringMachine::from_multi(&["abcbababccb"], Application::preset_one_tape()).unwrap();
        assert_eq!(tm.last().unwrap(), ["qz0"]);
    }

    #[test]
    fn test_one_tape_15() {
        let tm = TuringMachine::from_multi(&["aaabbb"], Application::preset_one_tape()).unwrap();
        assert_eq!(tm.last().unwrap(), ["qz0"]);
    }

    #[test]
    fn test_one_tape_16() {
        let tm = TuringMachine::from_multi(&["aaabbba"], Application::preset_one_tape()).unwrap();
        assert_eq!(tm.last().unwrap(), ["qz0"]);
    }

    #[test]
    fn test_one_tape_17() {
        let tm = TuringMachine::from_multi(&["aaabccc"], Application::preset_one_tape()).unwrap();
        assert_eq!(tm.last().unwrap(), ["qz1"]);
    }

    #[test]
    fn test_one_tape_18() {
        let tm = TuringMachine::from_multi(&["abbbbc"], Application::preset_one_tape()).unwrap();
        assert_eq!(tm.last().unwrap(), ["qz1"]);
    }

    #[test]
    fn test_one_tape_19() {
        let tm = TuringMachine::from_multi(&["abbbbcc"], Application::preset_one_tape()).unwrap();
        assert_eq!(tm.last().unwrap(), ["qz0"]);
    }

    #[test]
    fn test_multitape_1() {
        let tm = TuringMachine::from_multi(&[""; 2], Application::preset_multitape()).unwrap();
        assert_eq!(tm.last().unwrap()[0], "qz0");
    }

    #[test]
    fn test_multitape_2() {
        let alphabet = &['a', 'b', 'c'];
        let preset = Application::preset_multitape();
        for word in alphabet
            .get_exhaustive_words(3)
            .chain(alphabet.get_exhaustive_words(2))
            .chain(alphabet.get_exhaustive_words(1))
        {
            let tm = TuringMachine::from_multi(&[&word, ""], preset.clone()).unwrap();
            assert_eq!(tm.last().unwrap()[0], "qz0");
        }
    }

    #[test]
    fn test_multitape_3() {
        let tm =
            TuringMachine::from_multi(&["aabbcc", ""], Application::preset_multitape()).unwrap();
        assert_eq!(tm.last().unwrap()[0], "qz0");
    }

    #[test]
    fn test_multitape_4() {
        let tm =
            TuringMachine::from_multi(&["aabbbcc", ""], Application::preset_multitape()).unwrap();
        assert_eq!(tm.last().unwrap()[0], "qz1");
    }

    #[test]
    fn test_multitape_5() {
        let tm =
            TuringMachine::from_multi(&["aaabbccc", ""], Application::preset_multitape()).unwrap();
        assert_eq!(tm.last().unwrap()[0], "qz1");
    }

    #[test]
    fn test_multitape_6() {
        let tm =
            TuringMachine::from_multi(&["aaabbcacc", ""], Application::preset_multitape()).unwrap();
        assert_eq!(tm.last().unwrap()[0], "qz0");
    }

    #[test]
    fn test_multitape_7() {
        let tm =
            TuringMachine::from_multi(&["aaabbccbc", ""], Application::preset_multitape()).unwrap();
        assert_eq!(tm.last().unwrap()[0], "qz0");
    }

    #[test]
    fn test_multitape_8() {
        let tm =
            TuringMachine::from_multi(&["aaaacccca", ""], Application::preset_multitape()).unwrap();
        assert_eq!(tm.last().unwrap()[0], "qz0");
    }

    #[test]
    fn test_multitape_9() {
        let tm =
            TuringMachine::from_multi(&["aabbbbc", ""], Application::preset_multitape()).unwrap();
        assert_eq!(tm.last().unwrap()[0], "qz0");
    }

    #[test]
    fn test_multitape_10() {
        let tm =
            TuringMachine::from_multi(&["abbbbc", ""], Application::preset_multitape()).unwrap();
        assert_eq!(tm.last().unwrap()[0], "qz1");
    }

    #[test]
    fn test_multitape_11() {
        let tm =
            TuringMachine::from_multi(&["abbbbcc", ""], Application::preset_multitape()).unwrap();
        assert_eq!(tm.last().unwrap()[0], "qz0");
    }

    #[test]
    fn test_multitape_12() {
        let tm =
            TuringMachine::from_multi(&["aaabbbccc", ""], Application::preset_multitape()).unwrap();
        assert_eq!(tm.last().unwrap()[0], "qz0");
    }
}
