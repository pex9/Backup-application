use std::sync::mpsc::Sender;
use std::sync::{Arc, Condvar, Mutex};
use egui::Context;
use eframe::NativeOptions;

const APP_NAME: &str = "Emergency Backup";
struct ConfirmGui {
    choice: Sender<Choice>,
    controller: Arc<(Mutex<bool>, Condvar)>
}

pub enum Choice {
    Yes,
    No,
    CloseGui, //used to close the gui when user use gesture instead
}

impl ConfirmGui {
    pub fn new(choice: Sender<Choice>,controller: Arc<(Mutex<bool>, Condvar)>) -> Self {
        Self { choice,controller }
    }
}

impl eframe::App for ConfirmGui {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        //check if user instead use the gesture and close the gui
        {
            let (lock, _) = &*self.controller;
            let guard = lock.lock().unwrap();
            if *guard {
                _frame.close();
                self.choice.send(Choice::CloseGui).expect("TODO: panic message");
            }
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Do you want to start the  backup?");
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.centered_and_justified(|ui| {
                        if ui.button("Yes").clicked() {
                            _frame.close();
                            self.choice.send(Choice::Yes).expect("TODO: panic message");
                        }
                        ui.add_space(4.0);
                        if ui.button("No").clicked() {
                            _frame.close();
                            self.choice.send(Choice::No).expect("TODO: panic message");
                        }
                    });
                });
            });
        });
    }
}

pub fn run_confirm_gui(sender: Sender<Choice>,controller: Arc<(Mutex<bool>, Condvar)>) {
    let options = NativeOptions {
        initial_window_size: Some(egui::vec2(250.0, 140.0)),
        drag_and_drop_support: false,
        resizable: false,
        ..Default::default()
    };

    eframe::run_native(
        APP_NAME,
        options,
        Box::new(move |_cc| Box::new(ConfirmGui::new(sender,controller))),
    )
}
