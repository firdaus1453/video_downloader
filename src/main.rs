use eframe::egui;
use rfd::FileDialog;
use std::path::PathBuf;
use tokio::sync::mpsc;
use youtube_dl::YoutubeDl;
use std::thread;

struct VideoDownloaderApp {
    url: String,
    download_dir: Option<PathBuf>,
    status: String,
    is_downloading: bool,
    tx: mpsc::Sender<String>,
    rx: mpsc::Receiver<String>,
}

impl Default for VideoDownloaderApp {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel(100);
        let default_dir = dirs::download_dir();
        Self {
            url: String::new(),
            download_dir: default_dir,
            status: "Selesai memuat aplikasi. Siap mengunduh.".to_string(),
            is_downloading: false,
            tx,
            rx,
        }
    }
}

impl eframe::App for VideoDownloaderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Cek jika ada pembaruan status dari thread download
        while let Ok(msg) = self.rx.try_recv() {
            self.status = msg.clone();
            if msg == "Unduhan selesai!" || msg.starts_with("Error:") {
               self.is_downloading = false; 
            }
        }

        // Font and text size settings
        let mut style = (*ctx.style()).clone();
        style.text_styles.get_mut(&egui::TextStyle::Body).unwrap().size = 16.0;
        style.text_styles.get_mut(&egui::TextStyle::Button).unwrap().size = 18.0;
        style.text_styles.get_mut(&egui::TextStyle::Heading).unwrap().size = 24.0;
        ctx.set_style(style);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Aplikasi Pengunduh Video");
            ui.add_space(10.0);

            ui.label("Masukkan URL Video (misal: Youtube):");
            let url_input = ui.add_sized([ui.available_width(), 30.0], egui::TextEdit::singleline(&mut self.url).hint_text("https://youtu.be/..."));
            
            ui.add_space(15.0);

            ui.horizontal(|ui| {
                ui.label("Lokasi Unduhan: ");
                let dir_str = match &self.download_dir {
                    Some(path) => path.to_string_lossy().to_string(),
                    None => "Pilih Folder...".to_string(),
                };
                
                if ui.button("Pilih Folder").clicked() && !self.is_downloading {
                    if let Some(path) = FileDialog::new().pick_folder() {
                        self.download_dir = Some(path);
                    }
                }
                ui.label(dir_str);
            });

            ui.add_space(20.0);

            ui.separator();

            ui.add_space(20.0);

            ui.horizontal(|ui| {
                if self.is_downloading {
                    ui.add(egui::Spinner::new());
                    ui.label("Sedang mengunduh...");
                } else {
                    if ui.button("Unduh Video").clicked() {
                        if self.url.is_empty() {
                            self.status = "Error: URL tidak boleh kosong!".to_string();
                        } else if self.download_dir.is_none() {
                            self.status = "Error: Lokasi unduhan belum dipilih!".to_string();
                        } else {
                            self.is_downloading = true;
                            self.status = "Memulai proses unduhan...".to_string();
                            
                            let url = self.url.clone();
                            let download_dir = self.download_dir.clone().unwrap();
                            let tx = self.tx.clone();
                            let ctx_clone = ctx.clone();

                            // Spawn download process in a background thread
                            thread::spawn(move || {
                                let _ = tx.blocking_send("Mencetak info video...".to_string());
                                ctx_clone.request_repaint();

                                let run_dl = || -> Result<(), Box<dyn std::error::Error>> {
                                    YoutubeDl::new(&url)
                                        .download_to(&download_dir)?;
                                    Ok(())
                                };

                                match run_dl() {
                                    Ok(_) => {
                                        let _ = tx.blocking_send("Unduhan selesai!".to_string());
                                    }
                                    Err(e) => {
                                        let _ = tx.blocking_send(format!("Error: {}", e));
                                    }
                                }
                                ctx_clone.request_repaint();
                            });
                        }
                    }
                }
            });

            ui.add_space(20.0);

            // Status message
            if self.status.starts_with("Error") {
                ui.colored_label(egui::Color32::from_rgb(255, 100, 100), &self.status);
            } else if self.status == "Unduhan selesai!" {
                ui.colored_label(egui::Color32::from_rgb(100, 255, 100), &self.status);
            } else {
                ui.label(&self.status);
            }
        });
    }
}

fn load_icon(bytes: &[u8]) -> Option<egui::IconData> {
    let image = image::load_from_memory(bytes).ok()?.into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    Some(egui::IconData {
        rgba,
        width,
        height,
    })
}

fn main() -> eframe::Result<()> {
    let icon = load_icon(include_bytes!("../assets/icon.png"));
    
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([600.0, 350.0])
        .with_min_inner_size([400.0, 300.0]);
        
    if let Some(icon_data) = icon {
        viewport = viewport.with_icon(icon_data);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "Pengunduh Video Sederhana",
        options,
        Box::new(|_cc| Ok(Box::new(VideoDownloaderApp::default()))),
    )
}
