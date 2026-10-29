use eframe::egui;
use shared::{ContactStore, NewContact};

use crate::sqlite_store::SqliteStore;
mod sqlite_store;

#[cfg(test)]
mod store;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions::default();

    eframe::run_native("crm", native_options, Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc)))))
}

struct MyEguiApp {
    store: Box<dyn ContactStore>,
    search: String,
    form: NewContact,
    error: Option<String>,
    selected: Option<u32>,
}

impl MyEguiApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut store = Box::new(SqliteStore::open("crm.db").expect("failed to open database"));

        if store.list().is_empty() {
            let seed = [
                ("Dana", "Whitfield", "Gulf Valve Supply", "dana@gulfvalve.example"),
                ("Marcus", "Lee", "Bayou Fabrication", "mlee@bayoufab.example"),
                ("Priya", "Nair", "Delta Process Co.", "priya@deltaprocess.example"),
                ("Tom", "Boudreaux", "Atchafalaya Pumps", "tom@apumps.example"),
                ("Rachel", "Kim", "Southern Seals", "rkim@southernseals.example"),
                ("Luis", "Ortega", "Red Stick Instruments", "lortega@redstick.example"),
                ("Hannah", "Fontenot", "Pelican Piping", "hannah@pelicanpiping.example"),
                ("James", "Okafor", "Crescent Controls", "jokafor@crescentctl.example"),
                ("Mei", "Chen", "Levee Line Industrial", "mchen@leveeline.example"),
                ("Grant", "Thibodeaux", "Cypress Flow Systems", "grant@cypressflow.example"),
            ];

            for (first_name, last_name, company, email) in seed {
                store
                    .create(NewContact {
                        first_name: first_name.to_string(),
                        last_name: last_name.to_string(),
                        company: company.to_string(),
                        email: email.to_string(),
                        active: true,
                    })
                    .expect("seed data should be valid");
            }
        }

        Self {
            store,
            search: String::new(),
            form: NewContact::default(),
            error: None,
            selected: None,
        }
    }
}

impl eframe::App for MyEguiApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::right("new_contact").show(ui, |ui| {
            // TODO: probably needs to be more robust but for now this is fine
            if self.selected.is_none() {
                ui.heading("New Contact");
            } else {
                ui.heading("Edit Contact");
            }

            let first_name = ui.add(
                egui::TextEdit::singleline(&mut self.form.first_name).hint_text("First name: "),
            );
            let last_name = ui
                .add(egui::TextEdit::singleline(&mut self.form.last_name).hint_text("Last name: "));
            let email =
                ui.add(egui::TextEdit::singleline(&mut self.form.email).hint_text("Email: "));
            let company =
                ui.add(egui::TextEdit::singleline(&mut self.form.company).hint_text("Company: "));

            let enter_pressed: bool = ui.input(|i| i.key_pressed(egui::Key::Enter));
            let submitted_by_enter: bool = enter_pressed
                && (first_name.lost_focus()
                    || last_name.lost_focus()
                    || email.lost_focus()
                    || company.lost_focus());

            match self.selected {
                Some(id) => {
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() || submitted_by_enter {
                            match self.store.update(id, &self.form) {
                                Ok(_) => {
                                    self.form = NewContact::default();
                                    self.selected = None;
                                    self.error = None;
                                }
                                Err(e) => {
                                    self.error = Some(e.to_string());
                                }
                            }
                        }

                        if ui.button("Delete").clicked() {
                            match self.store.delete(id) {
                                Ok(_) => {
                                    self.form = NewContact::default();
                                    self.selected = None;
                                    self.error = None;
                                }
                                Err(e) => {
                                    self.error = Some(e.to_string());
                                }
                            }
                        }

                        if ui.button("Cancel").clicked() {
                            self.form = NewContact::default();
                            self.selected = None;
                            self.error = None;
                        }
                    });
                }
                None => {
                    if ui.button("Submit").clicked() || submitted_by_enter {
                        match self.store.create(self.form.clone()) {
                            Ok(_) => {
                                self.form = NewContact::default();
                                self.error = None;
                            }
                            Err(e) => {
                                self.error = Some(e.to_string());
                            }
                        }
                    }
                }
            }

            if let Some(msg) = &self.error {
                ui.label(msg);
            }
        });
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Contacts: ");

            ui.text_edit_singleline(&mut self.search);

            egui::ScrollArea::vertical().show(ui, |ui| {
                for contact in self.store.list() {
                    if contact.matches(&self.search) {
                        let is_selected: bool = self.selected == Some(contact.id);

                        let name_resp = ui.selectable_label(
                            is_selected,
                            egui::RichText::new(format!(
                                "{} {}",
                                contact.first_name, contact.last_name
                            ))
                            .size(20.0)
                            .strong(),
                        );
                        ui.label(&contact.company);
                        ui.label(&contact.email);
                        ui.separator();

                        if name_resp.clicked() {
                            self.selected = Some(contact.id);
                            self.form = NewContact {
                                first_name: contact.first_name.clone(),
                                last_name: contact.last_name.clone(),
                                company: contact.company.clone(),
                                email: contact.email.clone(),
                                active: contact.active,
                            };
                        }
                    }
                }
            });
        });
    }
}
