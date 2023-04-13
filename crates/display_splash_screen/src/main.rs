#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use blorb::{chunk::Chunk, types::BlorbType, BlorbReader};
use eframe::egui;
use egui_extras::RetainedImage;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(300.0, 900.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Show an image with eframe/egui",
        options,
        Box::new(|_cc| Box::<MyApp>::default()),
    )
}

struct MyApp {
    image: RetainedImage,
}

impl Default for MyApp {
    fn default() -> Self {
        let filename = std::env::args().nth(1).unwrap();
        let filedata = std::fs::read(filename).expect("unable to open file");
        let blorb = BlorbReader::new(filedata).expect("can't create reader");

        let Chunk::Frontispiece(fspc) = blorb
            .get_first_rsrc_by_type(BlorbType::Fspc)
            .expect("can't convert type");

        let image_data = blorb
            .get_resource_by_id(fspc)
            .expect("Missing ID for frontispiece");

        Self {
            image: RetainedImage::from_image_bytes("Frontispiece Image", image_data.bytes).unwrap(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.image.show(ui);
        });
    }
}
