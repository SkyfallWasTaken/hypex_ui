use hypex_ui::toasts;

fn main() -> eframe::Result<()> {
    re_log::setup_native_logging();

    let native_options = eframe::NativeOptions {
        app_id: Some("hypex_ui_example".to_owned()),

        initial_window_size: Some([1200.0, 800.0].into()),
        follow_system_theme: false,
        default_theme: eframe::Theme::Dark,

        #[cfg(target_os = "macos")]
        fullsize_content: hypex_ui::FULLSIZE_CONTENT,

        // Maybe hide the OS-specific "chrome" around the window:
        decorated: !hypex_ui::CUSTOM_WINDOW_DECORATIONS,
        // To have rounded corners we need transparency:
        transparent: hypex_ui::CUSTOM_WINDOW_DECORATIONS,

        ..Default::default()
    };

    eframe::run_native(
        "hypex_ui example app",
        native_options,
        Box::new(move |cc| {
            let hypex_ui = hypex_ui::HypexUi::load_and_apply(&cc.egui_ctx);
            Box::new(MyApp::new(hypex_ui))
        }),
    )
}

struct MyApp {
    _hypex_ui: hypex_ui::HypexUi,
    toasts: toasts::Toasts,

    /// Listens to the local text log stream
    text_log_rx: std::sync::mpsc::Receiver<re_log::LogMsg>,

    name: String,
    age: u32,
}

impl MyApp {
    fn new(hypex_ui: hypex_ui::HypexUi) -> Self {
        let (logger, text_log_rx) = re_log::ChannelLogger::new(re_log::LevelFilter::Info);
        re_log::add_boxed_logger(Box::new(logger)).unwrap();

        Self {
            _hypex_ui: hypex_ui,
            toasts: Default::default(),
            text_log_rx,

            name: String::from("Hypex"),
            age: 69,
        }
    }

    fn show_text_logs_as_notifications(&mut self) {
        while let Ok(re_log::LogMsg {
            level,
            target: _,
            msg,
        }) = self.text_log_rx.try_recv()
        {
            let kind = match level {
                re_log::Level::Error => toasts::ToastKind::Error,
                re_log::Level::Warn => toasts::ToastKind::Warning,
                re_log::Level::Info => toasts::ToastKind::Info,
                re_log::Level::Debug | re_log::Level::Trace => {
                    continue; // too spammy
                }
            };

            self.toasts.add(toasts::Toast {
                kind,
                text: msg,
                options: toasts::ToastOptions::with_ttl_in_seconds(4.0),
            });
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.show_text_logs_as_notifications();
        self.toasts.show(ctx);

        egui::gui_zoom::zoom_with_keyboard_shortcuts(ctx, frame.info().native_pixels_per_point);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello world!");
            ui.horizontal(|ui| {
                let name_label = ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.name)
                    .labelled_by(name_label.id);
            });
            ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            if ui.button("Click each year").clicked() {
                self.age += 1;
            }
            ui.label(format!("{} ({})", self.name, self.age));

            ui.horizontal(|ui| {
                if ui.button("Log info").clicked() {
                    re_log::info!(
                        "Here's a toast showing the info!\nThis is also logged to the console."
                    )
                }
                if ui.button("Log warning").clicked() {
                    re_log::warn!(
                        "Here's a toast showing the warning!\nThis is also logged to the console."
                    )
                }
                if ui.button("Log error").clicked() {
                    re_log::error!(
                        "Here's a toast showing the error!\nThis is also logged to the console."
                    )
                }
            });
            ui.end_row();
        });
    }
}
