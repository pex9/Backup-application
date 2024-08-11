use std::sync::mpsc::{Sender};
use egui::Context;
use eframe::NativeOptions;

pub struct ConfirmGui {
    choice: Sender<Choice>
}

pub enum Choice {
    Yes,
    No,
}

impl ConfirmGui {
    pub fn new(choice: Sender<Choice>) -> Self {
        Self { choice }
    }
}

impl eframe::App for ConfirmGui {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Do you want to start the  backup?");
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.centered_and_justified(|ui| {
                        if ui.button("Yes").clicked() {
                            self.choice.send(Choice::Yes).expect("TODO: panic message");

                        }
                        ui.add_space(4.0);
                        if ui.button("No").clicked() {
                            self.choice.send(Choice::No).expect("TODO: panic message");
                        }
                    });
                });
            });
        });
    }
}

pub fn run_confirm_gui(sender: Sender<Choice>) {

    let  options = NativeOptions {
        initial_window_size: Some(egui::vec2(250.0, 140.0)),
        drag_and_drop_support: false,
        resizable: false,
        ..Default::default()
    };

    eframe::run_native(
        "Back-up",
        options,
        Box::new(move |_cc| Box::new(ConfirmGui::new(sender))),
    )
}
