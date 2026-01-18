use eframe::egui;

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Mon Moteur Rust",
        options,
        Box::new(|_cc| Ok(Box::new(MonApp::default()))),
    )
    .unwrap();
}

#[derive(Default)]
struct MonApp {
    compteur: i32,
}

impl eframe::App for MonApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Interface de mon moteur");

            if ui.button("Clique-moi !").clicked() {
                self.compteur += 1;
            }

            ui.label(format!("Compteur : {}", self.compteur));
        });
    }
}
