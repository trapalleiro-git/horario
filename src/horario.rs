use chrono::{Datelike, Duration, NaiveTime, Utc};
use directories::ProjectDirs;
use eframe::{egui, epi};
use secfmt;
use serde::{Deserialize, Serialize};

const HM: &'static str = "%H : %M";

const SHORT_MONTH_NAMES: [&'static str; 13] = [
    "---", "Ene", "Feb", "Mar", "Abr", "May", "Jun", "Jul", "Ago", "Sep", "Oct", "Nov", "Dic",
];

const DAYS_WEEK_NAMES: [&'static str; 7] = [
    "    Lunes  ",
    "   Martes  ",
    " Miércoles ",
    "   Jueves  ",
    "  Viernes  ",
    "   Sabado  ",
    "   Domingo ",
];

const CONFIG_FIELDS: [&'static str; 4] = [
    "Saldo Semanal / 5:",
    "Obligatorio Tardes:  [ \u{2605} ]",
    "Tiempo a Recuperar:  [ \u{2691} ]",
    "Automático:",
];

const CONFIG_SALDO: [&'static str; 3] = ["\u{2795}", "\u{26f6}", "\u{2796}"];

#[derive(Clone, Debug)]
enum Menu {
    Horario,
    Configurar,
    About,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Datos {
    fichajes: Vec<String>,
    config: Vec<String>,
}

impl Default for Datos {
    fn default() -> Self {
        Self {
            fichajes: vec![String::new(); 20],
            config: vec![
                "   07 : 30".to_owned(),
                String::new(),
                String::new(),
                "false".to_owned(),
            ],
        }
    }
}

#[derive(Clone, Debug)]
pub struct Horario {
    datos: Datos,
    x: usize,
    check: bool,
    color: egui::Color32,
    salida_viernes: String,
    menu: Menu,
}

impl Default for Horario {
    fn default() -> Self {
        Self {
            datos: Datos::default(),
            x: 0,
            check: false,
            color: egui::Color32::GOLD,
            salida_viernes: String::new(),
            menu: Menu::Horario,
        }
    }
}

impl epi::App for Horario {
    fn name(&self) -> &str {
        "Horario"
    }

    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &mut epi::Frame<'_>,
        storage: Option<&dyn epi::Storage>,
    ) {
        if let Some(storage) = storage {
            self.datos = epi::get_value(storage, epi::APP_KEY).unwrap_or_default();
        }

        self.check = if self.datos.config[3] == "true" {
            true
        } else {
            false
        };
    }

    fn save(&mut self, storage: &mut dyn epi::Storage) {
        if self.check {
            self.datos.config[3] = String::from("true");
        } else {
            self.datos.config[3] = String::from("false");
        };

        epi::set_value(storage, epi::APP_KEY, &self.datos);
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        //ctx.set_visuals(eframe::egui::Visuals::dark());
        frame.set_window_size(egui::Vec2 { x: 610.0, y: 375.0 });

        let mut fonts = egui::FontDefinitions::default();
        fonts.family_and_size.insert(
            egui::TextStyle::Body,
            (egui::FontFamily::Proportional, 24.0),
        );
        fonts.family_and_size.insert(
            egui::TextStyle::Heading,
            (egui::FontFamily::Proportional, 20.0),
        );
        fonts.family_and_size.insert(
            egui::TextStyle::Button,
            (egui::FontFamily::Proportional, 18.0),
        );
        fonts.family_and_size.insert(
            egui::TextStyle::Small,
            (egui::FontFamily::Proportional, 17.0),
        );
        ctx.set_fonts(fonts);

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(3.0);
            egui::menu::bar(ui, |ui| {
                ui.with_layout(egui::Layout::left_to_right(), |ui| {
                    // Configurar
                    if ui.button(" \u{2692} ").clicked() {
                        self.menu = Menu::Configurar;
                    }
                    // Reset
                    if ui.button(" \u{21ba} ").clicked() {
                        self.menu = Menu::Horario;
                        self.datos.fichajes = vec![String::new(); 20];
                    }
                });

                ui.with_layout(egui::Layout::right_to_left(), |ui| {
                    // About
                    if ui.button(" ?").clicked() {
                        self.menu = Menu::About;
                    }
                });
            });
            ui.add_space(0.25);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(
                egui::Layout::from_main_dir_and_cross_align(
                    egui::Direction::TopDown,
                    egui::Align::Center,
                )
                .with_main_wrap(false)
                .with_cross_justify(true),
                |ui| match self.menu {
                    Menu::Horario => self.menu_horario(ui),
                    Menu::Configurar => self.menu_configurar(ui),
                    Menu::About => self.menu_about(ui),
                },
            );
        });
    }
}

impl Horario {
    fn check_fichaje(&mut self) {
        let from_str = NaiveTime::parse_from_str;
        let since = NaiveTime::signed_duration_since;
        let zero = from_str("00:00", "%H:%M").unwrap();
        let i = self.x;

        let t1 = since(
            from_str(self.datos.fichajes[i % 5].trim(), HM).unwrap_or(zero),
            zero,
        )
        .num_seconds();

        let t2 = since(
            from_str(self.datos.fichajes[i % 5 + 5].trim(), HM).unwrap_or(zero),
            zero,
        )
        .num_seconds();

        let t3 = since(
            from_str(self.datos.fichajes[i % 5 + 10].trim(), HM).unwrap_or(zero),
            zero,
        )
        .num_seconds();

        let t4 = since(
            from_str(self.datos.fichajes[i % 5 + 15].trim(), HM).unwrap_or(zero),
            zero,
        )
        .num_seconds();

        if t1 > t2 && t2 != 0 || t3 > t4 && t4 != 0 || t2 > t3 && t3 != 0 {
            self.datos.fichajes[i] = "".to_owned()
        }
    }

    fn calculo_saldo(&mut self) -> String {
        let from_str = NaiveTime::parse_from_str;
        let since = NaiveTime::signed_duration_since;
        let zero = from_str("00:00", "%H:%M").unwrap();
        let i = self.x;

        // -----------------------------------------------------------------------------------
        // ---      Calculo Saldo      -------------------------------------------------------
        // -----------------------------------------------------------------------------------

        let t = from_str(&self.datos.config[1].trim(), HM).unwrap_or(zero);
        let time_tardes = since(t, zero).num_seconds();
        let t = from_str(&self.datos.config[2].trim(), HM).unwrap_or(zero);
        let time_plus = since(t, zero).num_seconds();
        let t = from_str(&self.datos.config[0].trim(), HM).unwrap_or(zero);
        let time_job = since(t, zero).num_seconds() * 5 + time_plus;

        // -----  Dia  ----------------------------------------------------------------------

        let mut times_dia = Vec::new();

        let fic: Vec<&str> = self
            .datos
            .fichajes
            .iter()
            .map(|x| x.trim())
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

        if self.datos.fichajes[i % 5].trim() == ""
            && self.datos.fichajes[i % 5 + 5].trim() == ""
            && self.datos.fichajes[i % 5 + 10].trim() == ""
            && self.datos.fichajes[i % 5 + 15].trim() == ""
        {
            dia = since(t, zero).num_seconds();
        }

        // -----  Viernes  ----------------------------------------------------------------------

        let mut times_dia = Vec::new();

        times_dia.push([(&fic[4], &fic[9]), (&fic[14], &fic[19])]);

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

        // -----  Tardes  -------------------------------------------------------------------

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

        // -----  Total  --------------------------------------------------------------------

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
            if self.datos.fichajes[i % 5].trim() == ""
                && self.datos.fichajes[i % 5 + 5].trim() == ""
                && self.datos.fichajes[i % 5 + 10].trim() == ""
                && self.datos.fichajes[i % 5 + 15].trim() == ""
            {
                total += since(t, zero).num_seconds();
            }
        }

        // -----  Automático  -----------------------------------------------------------------------------

        if self.check == true && self.datos.fichajes[4].trim() != "" {
            self.datos.fichajes[14] = "".to_owned();
            self.datos.fichajes[19] = "".to_owned();

            let ent_vie = since(
                from_str(self.datos.fichajes[4].trim(), HM).unwrap_or(zero),
                zero,
            )
            .num_seconds();
            let mut sal_vie = ent_vie + (time_job - total + viernes);

            if i == 4 || i == 9 {
                dia = sal_vie - ent_vie;
            }

            if sal_vie > 86400 {
                sal_vie = sal_vie - 86400
            }
            let sal_vie = NaiveTime::from_num_seconds_from_midnight(sal_vie as u32, 0);

            self.salida_viernes = sal_vie.format("   %H : %M").to_string();

            total = time_job;
        }

        // -----------------------------------------------------------------------------------------------

        let dia = NaiveTime::from_num_seconds_from_midnight(dia as u32, 0);

        let mut txt_saldo: String = format!("{}", DAYS_WEEK_NAMES[i % 5]);
        txt_saldo = format!("{} {}", txt_saldo, dia.format(" %H : %M ").to_string());

        match total > time_job {
            true => {
                self.color = egui::Color32::GREEN;
                let time = secfmt::from(total as u64 - time_job as u64);
                txt_saldo = format!(
                    "{}          {}          [  {}  ]",
                    txt_saldo,
                    CONFIG_SALDO[0],
                    format!("{}h  {}m", time.hours, time.minutes)
                );
            }
            false => match total == time_job {
                true => {
                    self.color = egui::Color32::GOLD;
                    let time = secfmt::from(total as u64);
                    txt_saldo = format!(
                        "{}          {}          [  {}  ]",
                        txt_saldo,
                        CONFIG_SALDO[1],
                        format!("{}h  {}m", time.days * 24 + time.hours, time.minutes)
                    );
                }
                false => {
                    self.color = egui::Color32::RED;
                    let time = secfmt::from(time_job as u64 - total as u64);
                    txt_saldo = format!(
                        "{}          {}          [  {}  ]",
                        txt_saldo,
                        CONFIG_SALDO[2],
                        format!("{}h  {}m", time.hours, time.minutes)
                    );
                }
            },
        }

        if time_tardes > tardes {
            txt_saldo = format!("{} {}", txt_saldo, "\u{2605}");
        }
        if time_plus > 0 {
            txt_saldo = format!("{} {}", txt_saldo, "\u{2691}");
        }

        txt_saldo
    }

    fn menu_configurar(&mut self, ui: &mut egui::Ui) {
        let zero = NaiveTime::parse_from_str("00:00", "%H:%M").unwrap();

        ui.add_space(5.0);
        ui.colored_label(egui::Color32::DEBUG_COLOR, "Configuración");
        ui.add_space(30.0);

        egui::Grid::new("config")
            .num_columns(4)
            .min_col_width(103.0)
            .spacing([15.0, 20.0])
            .show(ui, |ui| {
                for i in 0..4 {
                    ui.label("");
                    ui.add(egui::Label::new(CONFIG_FIELDS[i]).text_style(egui::TextStyle::Heading));
                    if i == 3 {
                        ui.checkbox(&mut self.check, "");
                    } else {
                        let text = ui.text_edit_singleline(&mut self.datos.config[i]);

                        let mut t: NaiveTime =
                            NaiveTime::parse_from_str(self.datos.config[i].as_str().trim(), HM)
                                .unwrap_or(zero);
                        if t == zero {
                            t = NaiveTime::parse_from_str(
                                self.datos.config[i].as_str().trim(),
                                "%H%M",
                            )
                            .unwrap_or(zero);
                        }

                        if text.gained_focus() {
                            self.datos.config[i] = if t == zero {
                                "".to_owned()
                            } else {
                                t.format("    %H%M").to_string()
                            }
                        }

                        if ui.input().key_pressed(egui::Key::Enter)
                            || ui.input().key_pressed(egui::Key::Tab)
                            || text.clicked_elsewhere()
                        {
                            self.datos.config[i] = if t == zero {
                                "".to_owned()
                            } else {
                                t.format("   %H : %M").to_string()
                            }
                        }
                    }
                    ui.end_row();
                }

                ui.label("");
                ui.label("");
                if ui
                    .add(egui::Button::new("  Aceptar  ").text_style(egui::TextStyle::Small))
                    .clicked()
                {
                    if self.check == true && self.datos.fichajes[4].trim() != "" {
                        self.calculo_saldo();
                        self.datos.fichajes[9] = self.salida_viernes.to_owned();
                    }
                    if self.datos.config[0] == "".to_owned() {
                        self.datos.config[0] = "   07 : 30".to_owned();
                    }
                    self.menu = Menu::Horario;
                };
            });
    }

    fn menu_horario(&mut self, ui: &mut egui::Ui) {
        let zero = NaiveTime::parse_from_str("00:00", "%H:%M").unwrap();

        ui.add_space(5.0);
        ui.colored_label(egui::Color32::LIGHT_BLUE, get_week());
        ui.add_space(21.0);

        let dt = Utc::now();
        let num_day_week = dt.weekday().number_from_monday(); // Mon=1, ..., Vie=5, Sat=6, Sun=7
        let ndw = num_day_week as usize;

        egui::Grid::new("fichajes")
            .spacing(egui::Vec2::new(20.0, 15.0))
            .min_col_width(103.0)
            .show(ui, |ui| {
                for i in 0..5 {
                    if i == (ndw - 1) {
                        ui.add(
                            egui::Label::new(format!("  {}", DAYS_WEEK_NAMES[i]).as_str())
                                .text_style(egui::TextStyle::Heading)
                                .text_color(egui::Color32::KHAKI),
                        );
                    } else {
                        ui.add(
                            egui::Label::new(format!("  {}", DAYS_WEEK_NAMES[i]).as_str())
                                .text_style(egui::TextStyle::Heading),
                        );
                    }
                }

                for (i, _) in self.datos.fichajes.clone().iter().enumerate() {
                    if i % 5 == 0 {
                        ui.end_row();
                    }

                    let text = ui.text_edit_singleline(&mut self.datos.fichajes[i]);

                    let mut t: NaiveTime =
                        NaiveTime::parse_from_str(self.datos.fichajes[i].as_str().trim(), HM)
                            .unwrap_or(zero);

                    if t == zero {
                        t = NaiveTime::parse_from_str(
                            self.datos.fichajes[i].as_str().trim(),
                            "%H%M",
                        )
                        .unwrap_or(zero);
                    }

                    if text.gained_focus() {
                        self.x = i;
                        self.datos.fichajes[i] = if t == zero {
                            "".to_owned()
                        } else {
                            t.format("    %H%M").to_string()
                        };
                    }

                    if ui.input().key_pressed(egui::Key::Enter)
                        || ui.input().key_pressed(egui::Key::Tab)
                        || text.clicked_elsewhere()
                    {
                        self.datos.fichajes[i] = if t == zero {
                            "".to_owned()
                        } else {
                            t.format("   %H : %M").to_string()
                        };

                        self.check_fichaje();
                    }
                }
            });

        ui.add_space(30.0);
        ui.add(
            egui::Label::new(self.calculo_saldo())
                .text_style(egui::TextStyle::Heading)
                .text_color(self.color),
        );

        if self.check == true && self.datos.fichajes[4].trim() != "" {
            if ui.input().key_pressed(egui::Key::Enter) || ui.input().key_pressed(egui::Key::Tab) {
                self.datos.fichajes[9] = self.salida_viernes.to_owned();
            }
        }

        if self.check == true && self.datos.fichajes[4].trim() == "" {
            if ui.input().key_pressed(egui::Key::Enter) || ui.input().key_pressed(egui::Key::Tab) {
                self.datos.fichajes[9] = "".to_owned();
            }
        }
    }

    fn menu_about(&mut self, ui: &mut egui::Ui) {
        ui.add_space(50.0);
        ui.colored_label(egui::Color32::LIGHT_BLUE, " \u{00a9} 2021 Trapalleiro");

        // Called on shutdown, and perhaps at regular intervals. Allows you to save state.
        // Only called when the "persistence" feature is enabled.
        //
        // On web the states is stored to "Local Storage".
        // On native the path is picked using [`directories_next::ProjectDirs::data_dir`]
        // (https://docs.rs/directories-next/2.0.0/directories_next/struct.ProjectDirs.html#method.data_dir) which is:
        // * Linux:   `/home/UserName/.local/share/APPNAME`
        // * macOS:   `/Users/UserName/Library/Application Support/APPNAME`
        // * Windows: `C:\Users\UserName\AppData\Roaming\APPNAME`
        //
        // where `APPNAME` is what is returned by [`Self::name()`].

        ui.add_space(20.0);
        if let Some(proj_dirs) = ProjectDirs::from("", "", epi::App::name(self)) {
            ui.colored_label(egui::Color32::GOLD, "Datos\n");
            ui.colored_label(
                egui::Color32::DEBUG_COLOR,
                proj_dirs.data_dir().to_string_lossy(),
            );
        }

        ui.add_space(50.0);
        if ui
            .add(egui::Button::new("Aceptar").text_style(egui::TextStyle::Small))
            .clicked()
        {
            self.menu = Menu::Horario;
        };
    }
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
