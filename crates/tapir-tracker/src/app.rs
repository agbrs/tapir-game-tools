use crate::widgets::piano;

pub struct TapirTrackerApp {}

impl TapirTrackerApp {
    pub fn new(cc: &eframe::CreationContext<'_>, _filename: Option<String>) -> Self {
        catppuccin_egui::set_theme(&cc.egui_ctx, catppuccin_egui::FRAPPE);

        Self {}
    }
}

impl eframe::App for TapirTrackerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        use egui::{
            menu, scroll_area::ScrollBarVisibility, Align, CentralPanel, Layout, ScrollArea,
            SidePanel, TopBottomPanel,
        };

        TopBottomPanel::top("menu").show(ctx, |ui| {
            menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    let _ = ui.button("New");
                });

                ui.menu_button("Help", |ui| {
                    let _ = ui.button("About");
                })
            });
        });

        SidePanel::left("controls_and_sections").show(ctx, |ui| {
            ui.label("Play, pause etc");
            ui.label("All the parts of your track");
        });

        TopBottomPanel::top("instruments").show(ctx, |ui| {
            ui.with_layout(
                Layout::with_main_justify(Layout::left_to_right(Align::LEFT), true),
                |ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("Instrument 1");
                            ui.label("Instrument 2");
                        });

                        ui.horizontal(|ui| {
                            ui.label("Channel 1");
                            ui.label("Channel 2");
                            ui.label("Channel 3");
                            ui.label("Channel 4");
                            ui.label("Channel 5");
                        });
                    });

                    ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                        let _ = ui.button("Settings");
                    });
                },
            );
        });

        TopBottomPanel::bottom("spectrum").show(ctx, |ui| {
            ui.label("Resulting channel display");
        });

        CentralPanel::default().show(ctx, |ui| {
            ScrollArea::both()
                .scroll_bar_visibility(ScrollBarVisibility::AlwaysVisible)
                .show(ui, |ui| {
                    piano(ui);
                });
        });
    }
}
