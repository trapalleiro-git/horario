#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(windows, windows_subsystem = "windows")]

use chrono::{Datelike, Duration, NaiveTime, Utc};
use directories_next::ProjectDirs;
use eframe::egui::{
    menu, vec2, Button, CentralPanel, Color32, Context, FontId, Grid, Key, Label, Layout, Response,
    RichText, Sense, TextEdit, TopBottomPanel, Ui, Visuals,
};

use eframe::{get_value, run_native, set_value, App, Frame, NativeOptions, Storage, APP_KEY};

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::error::Error;
use std::usize;

const APPNAME: &str = "Horario";

const HM: &str = "%H%M";

const SHORT_MONTH_NAMES: [&str; 13] = [
    "---", "Ene", "Feb", "Mar", "Abr", "May", "Jun", "Jul", "Ago", "Sep", "Oct", "Nov", "Dic",
];

const DAYS_WEEK_NAMES: [&str; 7] = [
    "Lunes",
    "Martes",
    "Mi\u{e9}rcoles",
    "Jueves",
    "Viernes",
    "Sabado",
    "Domingo",
];

const CONFIG_FIELDS: [&str; 4] = [
    "Saldo Semanal / 5:",
    "Obligatorio Tardes:  [ \u{2605} ]",
    "Tiempo a Recuperar:  [ \u{2691} ]",
    "Autom\u{e1}tico:",
];

const CONFIG_SALDO: [&str; 3] = ["\u{26f6}", "\u{2796}", "\u{2795}"];

#[derive(Clone, Debug)]
enum Menu {
    Horario,
    Configurar,
    About,
}

impl Default for Menu {
    fn default() -> Self {
        Self::Horario
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct Cell {
    #[serde(skip)]
    is_edit: bool,
    cell: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Datos {
    fichajes: Vec<Cell>,
    config: Vec<Cell>,
}

impl Default for Datos {
    fn default() -> Self {
        let iter = (0..20).map(|_a| Cell {
            is_edit: false,
            cell: String::new(),
        });

        Self {
            fichajes: Vec::from_iter(iter),
            config: vec![
                Cell {
                    is_edit: false,
                    cell: "      0730".to_owned(),
                },
                Cell {
                    is_edit: false,
                    cell: String::new(),
                },
                Cell {
                    is_edit: false,
                    cell: String::new(),
                },
                Cell {
                    is_edit: false,
                    cell: "false".to_owned(),
                },
            ],
        }
    }
}

#[derive(Clone, Debug)]
struct Horario {
    datos: Datos,
    x: usize,
    check: bool,
    menu: Menu,
}

impl Default for Horario {
    fn default() -> Self {
        Self {
            datos: Datos::default(),
            x: 0,
            check: false,
            menu: Menu::default(),
        }
    }
}

impl Horario {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(Visuals::dark());

        let mut data = Self::default();
        if let Some(storage) = cc.storage {
            data.datos = get_value(storage, APP_KEY).unwrap_or_default();
            data.check = data.datos.config[3].cell == "true";
        }
        data
    }

    #[allow(clippy::collapsible_if)]
    fn menu_horario(&mut self, ui: &mut Ui) {
        let zero = NaiveTime::parse_from_str("0000", HM).unwrap();

        ui.vertical_centered(|ui| {
            ui.add_space(15.);

            ui.add(Label::new(
                RichText::new(get_week())
                    .color(Color32::DEBUG_COLOR)
                    .font(FontId::proportional(24.)),
            ));

            ui.add_space(5.);

            let dt = Utc::now();

            // Mon=1, ..., Vie=5, Sat=6, Sun=7
            let num_day_week = dt.weekday().number_from_monday();
            let ndw = num_day_week as usize;

            ui.set_max_width(500.);

            ui.horizontal(|ui| {
                for (i, item) in DAYS_WEEK_NAMES.iter().enumerate().take(5) {
                    let color = if (ndw - 1) == i {
                        Color32::KHAKI
                    } else {
                        Color32::LIGHT_BLUE
                    };

                    let label = ui.add_sized(
                        [110., 50.],
                        Label::new(
                            RichText::new(format!("{}", *item))
                                .color(color)
                                .font(FontId::proportional(20.)),
                        )
                        .sense(Sense::click()),
                    );

                    if label.clicked() {
                        self.x = i;
                    }
                }
            });

            let mut index_cell: usize = 0;

            for row in self.datos.fichajes.chunks_mut(5) {
                ui.horizontal(|ui| {
                    for cell in row.iter_mut() {
                        let t: NaiveTime =
                            NaiveTime::parse_from_str(cell.cell.trim(), HM).unwrap_or(zero);

                        let txt_button = if t != zero {
                            t.format("%H : %M").to_string()
                        } else {
                            "".to_string()
                        };

                        let cool_button = create_cool_button(ui, &txt_button);

                        ui.allocate_ui_at_rect(cool_button.rect, |ui| {
                            let text = ui.add_visible(
                                cell.is_edit,
                                TextEdit::singleline(&mut cell.cell)
                                    .font(FontId::proportional(20.))
                                    .cursor_at_end(true),
                            );

                            if cool_button.clicked() {
                                if !cell.is_edit {
                                    cell.is_edit = true;

                                    self.x = index_cell;
                                    text.request_focus();
                                }
                            }

                            if ui.input().key_pressed(Key::Enter)
                                || ui.input().key_pressed(Key::Tab)
                                || text.clicked_elsewhere()
                            {
                                let t: NaiveTime =
                                    NaiveTime::parse_from_str(cell.cell.trim(), HM).unwrap_or(zero);

                                cell.cell = if t != zero {
                                    t.format("      %H%M").to_string()
                                } else {
                                    "      ".to_string()
                                };
                                cell.is_edit = false;
                            }
                        });

                        index_cell = index_cell + 1;
                    }
                });
            }

            if !self.datos.fichajes[self.x].is_edit {
                self.check_fichaje()
            };

            ui.add_space(30.);

            ui.label(self.calculo_saldo());
        });
    }

    fn menu_configurar(&mut self, ui: &mut Ui) {
        let zero = NaiveTime::parse_from_str("0000", HM).unwrap();

        ui.vertical_centered(|ui| {
            ui.add_space(15.);
            ui.label(
                RichText::new("Configuraci\u{f3}n")
                    .color(Color32::DEBUG_COLOR)
                    .font(FontId::proportional(24.)),
            );
            ui.add_space(20.);

            Grid::new("config")
                .num_columns(4)
                .min_col_width(103.)
                .spacing([15., 20.])
                .show(ui, |ui| {
                    let mut index_cell: usize = 0;

                    for cell in self.datos.config.iter_mut() {
                        ui.label("");
                        ui.add(Label::new(
                            RichText::new(CONFIG_FIELDS[index_cell])
                                .font(FontId::proportional(20.)),
                        ));

                        if index_cell == 3 {
                            ui.checkbox(&mut self.check, "");
                        } else {
                            let t: NaiveTime =
                                NaiveTime::parse_from_str(cell.cell.trim(), HM).unwrap_or(zero);

                            let txt_button = if t != zero {
                                t.format("%H : %M").to_string()
                            } else {
                                "".to_string()
                            };

                            let cool_button = create_cool_button(ui, &txt_button);

                            ui.allocate_ui_at_rect(cool_button.rect, |ui| {
                                let text = ui.add_visible(
                                    cell.is_edit,
                                    TextEdit::singleline(&mut cell.cell)
                                        .font(FontId::proportional(20.))
                                        .cursor_at_end(true),
                                );

                                if cool_button.clicked() {
                                    if !cell.is_edit {
                                        cell.is_edit = true;
                                        text.request_focus();
                                    }
                                }

                                if ui.input().key_pressed(Key::Enter)
                                    || ui.input().key_pressed(Key::Tab)
                                    || text.clicked_elsewhere()
                                {
                                    let t: NaiveTime =
                                        NaiveTime::parse_from_str(cell.cell.trim(), HM)
                                            .unwrap_or(zero);

                                    cell.cell = if t != zero {
                                        t.format("      %H%M").to_string()
                                    } else {
                                        "      ".to_string()
                                    };
                                    cell.is_edit = false;
                                }
                            });
                        }

                        ui.label("");
                        ui.end_row();
                        index_cell = index_cell + 1;
                    }
                });

            ui.add_space(30.);

            let button = Button::new(RichText::new("Aceptar").font(FontId::proportional(17.)));
            if ui.add_sized([100., 25.], button).clicked() {
                self.menu = Menu::Horario;
            };
        });
    }

    fn menu_about(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(50.);

            ui.label(
                RichText::new(" \u{00a9} 2022 Trapalleiro")
                    .font(FontId::proportional(24.))
                    .color(Color32::LIGHT_BLUE),
            );

            // Called on shutdown, and perhaps at regular intervals.
            // Allows you to save state.

            // Only called when the “persistence” feature is enabled.

            // On web the state is stored to “Local Storage”.
            // On native the path is picked using directories_next::ProjectDirs::data_dir which is:

            //     Linux: /home/UserName/.local/share/APPNAME
            //         macOS: /Users/UserName/Library/Application Support/APPNAME
            //             Windows: C:\Users\UserName\AppData\Roaming\APPNAME

            //             where APPNAME is what is given to eframe::run_native.

            ui.add_space(20.);

            if let Some(proj_dirs) = ProjectDirs::from("", "", APPNAME) {
                ui.label(
                    RichText::new("Datos\n")
                        .font(FontId::proportional(24.))
                        .color(Color32::GOLD),
                );
                ui.label(
                    RichText::new(proj_dirs.data_dir().to_string_lossy())
                        .font(FontId::proportional(20.))
                        .color(Color32::DEBUG_COLOR),
                );
            }

            ui.add_space(50.);

            let button = Button::new(RichText::new("Aceptar").font(FontId::proportional(17.)));
            if ui.add_sized([100., 25.], button).clicked() {
                self.menu = Menu::Horario;
            };
        });
    }

    // --------------------------------------------------------------------------------------------

    fn check_fichaje(&mut self) {
        let from_str = NaiveTime::parse_from_str;
        let since = NaiveTime::signed_duration_since;
        let zero = from_str("0000", HM).unwrap();
        let i = self.x;

        let t1 = since(
            from_str(self.datos.fichajes[i % 5].cell.trim(), HM).unwrap_or(zero),
            zero,
        )
        .num_seconds();

        let t2 = since(
            from_str(self.datos.fichajes[i % 5 + 5].cell.trim(), HM).unwrap_or(zero),
            zero,
        )
        .num_seconds();

        let t3 = since(
            from_str(self.datos.fichajes[i % 5 + 10].cell.trim(), HM).unwrap_or(zero),
            zero,
        )
        .num_seconds();

        let t4 = since(
            from_str(self.datos.fichajes[i % 5 + 15].cell.trim(), HM).unwrap_or(zero),
            zero,
        )
        .num_seconds();

        if t1 > t2 && t2 != 0 || t3 > t4 && t4 != 0 || t2 > t3 && t3 != 0 {
            self.datos.fichajes[i].cell = "      ".to_owned();
        }
    }

    #[allow(
        clippy::too_many_lines,
        clippy::similar_names,
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation
    )]
    fn calculo_saldo(&mut self) -> RichText {
        let from_str = NaiveTime::parse_from_str;
        let since = NaiveTime::signed_duration_since;
        let zero = from_str("0000", HM).unwrap();
        let i = self.x;

        // +--------------------------------------------------------------------------------------+
        // +++      Calculo Saldo                                                               +++
        // +--------------------------------------------------------------------------------------+

        let t = from_str(self.datos.config[1].cell.trim(), HM).unwrap_or(zero);
        let time_tardes = since(t, zero).num_seconds();
        let t = from_str(self.datos.config[2].cell.trim(), HM).unwrap_or(zero);
        let time_plus = since(t, zero).num_seconds();
        let t = from_str(self.datos.config[0].cell.trim(), HM).unwrap_or(zero);
        let time_job = since(t, zero).num_seconds() * 5 + time_plus;

        // -----  Dia  ----------------------------------------------------------------------------

        let mut times_dia = Vec::new();

        let fic: Vec<&str> = self
            .datos
            .fichajes
            .iter()
            .map(|x| x.cell.trim())
            .collect::<Vec<&str>>();

        times_dia.push([
            (&fic[i % 5], &fic[i % 5 + 5]),
            (&fic[i % 5 + 10], &fic[i % 5 + 15]),
        ]);

        let mut dia = times_dia
            .iter()
            .map(|day| {
                day.iter()
                    .map(|(t1, t2)| {
                        let start_time = from_str(t1, HM).unwrap_or(zero);
                        let end_time = from_str(t2, HM).unwrap_or(zero);
                        match since(end_time, start_time).num_seconds() {
                            x if x > 0 && start_time != zero => x,
                            _ => 0,
                        }
                    })
                    .collect::<Vec<i64>>()
                    .iter()
                    .sum::<i64>()
            })
            .collect::<Vec<i64>>()
            .iter()
            .sum::<i64>();

        if self.datos.fichajes[i % 5].cell.trim() == ""
            && self.datos.fichajes[i % 5 + 5].cell.trim() == ""
            && self.datos.fichajes[i % 5 + 10].cell.trim() == ""
            && self.datos.fichajes[i % 5 + 15].cell.trim() == ""
        {
            dia = since(t, zero).num_seconds();
        }

        // -----  Viernes  ------------------------------------------------------------------------

        let times_dia = vec![[(&fic[4], &fic[9]), (&fic[14], &fic[19])]];

        let viernes = times_dia
            .iter()
            .map(|day| {
                day.iter()
                    .map(|(t1, t2)| {
                        let start_time = from_str(t1, HM).unwrap_or(zero);
                        let end_time = from_str(t2, HM).unwrap_or(zero);
                        match since(end_time, start_time).num_seconds() {
                            x if x > 0 && start_time != zero => x,
                            _ => 0,
                        }
                    })
                    .collect::<Vec<i64>>()
                    .iter()
                    .sum::<i64>()
            })
            .collect::<Vec<i64>>()
            .iter()
            .sum::<i64>();

        // -----  Tardes  -------------------------------------------------------------------------

        let mut times_tardes = Vec::new();

        for i in 0..5 {
            times_tardes.push([(&fic[i + 10], &fic[i + 15])]);
        }

        let tardes = times_tardes
            .iter()
            .map(|tarde_times| {
                tarde_times
                    .iter()
                    .map(|(t1, t2)| {
                        let start_time = from_str(t1, HM).unwrap_or(zero);
                        let end_time = from_str(t2, HM).unwrap_or(zero);
                        match since(end_time, start_time).num_seconds() {
                            x if x > 0 && start_time != zero => x,
                            _ => 0,
                        }
                    })
                    .collect::<Vec<i64>>()
                    .iter()
                    .sum::<i64>()
            })
            .collect::<Vec<i64>>()
            .iter()
            .sum::<i64>();

        // -----  Total  --------------------------------------------------------------------------

        let mut times = Vec::new();

        for i in 0..5 {
            times.push([(&fic[i], &fic[i + 5]), (&fic[i + 10], &fic[i + 15])]);
        }

        let mut total = times
            .iter()
            .map(|day_times| {
                day_times
                    .iter()
                    .map(|(t1, t2)| {
                        let start_time = from_str(t1, HM).unwrap_or(zero);
                        let end_time = from_str(t2, HM).unwrap_or(zero);
                        match since(end_time, start_time).num_seconds() {
                            x if x > 0 && start_time != zero => x,
                            _ => 0,
                        }
                    })
                    .collect::<Vec<i64>>()
                    .iter()
                    .sum::<i64>()
            })
            .collect::<Vec<i64>>()
            .iter()
            .sum::<i64>();

        for i in 0..5 {
            if self.datos.fichajes[i % 5].cell.trim() == ""
                && self.datos.fichajes[i % 5 + 5].cell.trim() == ""
                && self.datos.fichajes[i % 5 + 10].cell.trim() == ""
                && self.datos.fichajes[i % 5 + 15].cell.trim() == ""
            {
                total += since(t, zero).num_seconds();
            }
        }

        // -----  Automático  --------------------------------------------------------------------

        if self.check {
            if self.datos.fichajes[4].cell.trim() != "" {
                self.datos.fichajes[14].cell = "".to_owned();
                self.datos.fichajes[19].cell = "".to_owned();

                let ent_vie = since(
                    from_str(self.datos.fichajes[4].cell.trim(), HM).unwrap_or(zero),
                    zero,
                )
                .num_seconds();
                let mut sal_vie = ent_vie + (time_job - total + viernes);

                if i == 4 || i == 9 {
                    dia = sal_vie - ent_vie;
                }

                if sal_vie > 86400 {
                    sal_vie -= 86400;
                }

                let sal_vie = NaiveTime::from_num_seconds_from_midnight(sal_vie as u32, 0);

                self.datos.fichajes[9].cell = sal_vie.format("      %H%M").to_string();

                total = time_job;
            } else {
                self.datos.fichajes[9].cell = "".to_owned();
            }
        }

        // -----  get RichText Saldo  -------------------------------------------------------------

        let dia = NaiveTime::from_num_seconds_from_midnight(dia as u32, 0);

        let mut txt_saldo: String = DAYS_WEEK_NAMES[i % 5].to_string();
        txt_saldo = format!("{} {}", txt_saldo, dia.format(" %H : %M ").to_string());

        let color: Color32;

        match total.cmp(&time_job) {
            Ordering::Equal => {
                color = Color32::GOLD;
                let time = secfmt::from(total as u64);
                txt_saldo = format!(
                    "{}          {}          [  {}  ]",
                    txt_saldo,
                    CONFIG_SALDO[0],
                    format!("{}h  {}m", time.days * 24 + time.hours, time.minutes)
                );
            }
            Ordering::Less => {
                color = Color32::RED;
                let time = secfmt::from(time_job as u64 - total as u64);
                txt_saldo = format!(
                    "{}          {}          [  {}  ]",
                    txt_saldo,
                    CONFIG_SALDO[1],
                    format!("{}h  {}m", time.hours, time.minutes)
                );
            }
            Ordering::Greater => {
                color = Color32::GREEN;
                let time = secfmt::from(total as u64 - time_job as u64);
                txt_saldo = format!(
                    "{}          {}          [  {}  ]",
                    txt_saldo,
                    CONFIG_SALDO[2],
                    format!("{}h  {}m", time.hours, time.minutes)
                );
            }
        }

        if time_tardes > tardes {
            txt_saldo = format!("{} {}", txt_saldo, "\u{2605}");
        }
        if time_plus > 0 {
            txt_saldo = format!("{} {}", txt_saldo, "\u{2691}");
        }

        RichText::new(txt_saldo)
            .font(FontId::proportional(23.))
            .color(color)
    }
}

impl App for Horario {
    fn save(&mut self, storage: &mut dyn Storage) {
        self.datos.config[3].cell = self.check.to_string();
        set_value(storage, APP_KEY, &self.datos);
    }

    fn persist_native_window(&self) -> bool {
        false
    }

    fn persist_egui_memory(&self) -> bool {
        false
    }

    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(3.0);
            menu::bar(ui, |ui| {
                ui.with_layout(Layout::left_to_right(), |ui| {
                    // Configurar
                    if ui.button(" \u{2692} ").clicked() {
                        self.menu = Menu::Configurar;
                    }
                    // Reset
                    if ui.button(" \u{21ba} ").clicked() {
                        self.menu = Menu::Horario;

                        let iter = (0..20).map(|_a| Cell {
                            is_edit: false,
                            cell: String::new(),
                        });

                        self.datos.fichajes = Vec::from_iter(iter);
                    }
                });

                ui.with_layout(Layout::right_to_left(), |ui| {
                    // About
                    if ui.button(" ?").clicked() {
                        self.menu = Menu::About;
                    }
                });
            });
            ui.add_space(0.25);
        });

        CentralPanel::default().show(ctx, |ui| match self.menu {
            Menu::Horario => self.menu_horario(ui),
            Menu::Configurar => self.menu_configurar(ui),
            Menu::About => self.menu_about(ui),
        });
    }
}

fn create_cool_button(ui: &mut Ui, cell: &str) -> Response {
    let button =
        Label::new(RichText::new(cell).font(FontId::proportional(20.))).sense(Sense::click());
    ui.add_sized([110., 30.], button)
}

fn get_week() -> String {
    let dt = Utc::now();
    let num_day_week = dt.weekday().number_from_monday(); // Mon=1, ..., Vie=5, Sat=6, Sun=7

    let lu = dt - Duration::days((num_day_week - 1).into());
    let vi = lu + Duration::days(4);

    format!(
        "{:02} {} {}        \u{2219}        {:02} {} {}",
        lu.day(),
        SHORT_MONTH_NAMES[lu.month() as usize],
        lu.year(),
        vi.day(),
        SHORT_MONTH_NAMES[vi.month() as usize],
        vi.year()
    )
}

fn main() -> Result<(), Box<dyn Error>> {
    let options = NativeOptions {
        initial_window_size: Some(vec2(600., 360.)),
        resizable: false,
        ..Default::default()
    };

    run_native(APPNAME, options, Box::new(|cc| Box::new(Horario::new(cc))));
}
/*

//-----------------------------------------------------------------------------------------------//
//                                                                                               //
//-----------------------------------------------------------------------------------------------//

[package]
name = "horario"
version = "0.1.0"
edition = "2021"
build = "build.rs"

[dependencies]
eframe = { version = "0.18.0", features = ["persistence"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = "0.4.19"
directories-next = "2.0.0"
secfmt = "0.1.1"

[target.'cfg(windows)'.build-dependencies]
winres = "0.1"

[profile.release]
opt-level = 3
debug = false
debug-assertions = false
overflow-checks = false
lto = true
panic = 'abort'
incremental = false
codegen-units = 1
rpath = false


// -----   build.rs   ---------------------------------------------------------------------------//

use std::io;
#[cfg(windows)]
use winres::WindowsResource;

fn main() -> io::Result<()> {
    #[cfg(windows)]
    {
        WindowsResource::new()
            // This path can be absolute, or relative to your crate root.
            .set_icon("assets/ficha.ico")
            .compile()?;
    }
    Ok(())
}

*/
