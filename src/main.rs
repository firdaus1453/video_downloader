use eframe::egui;
use rfd::FileDialog;
use std::path::PathBuf;
use tokio::sync::mpsc;
use std::thread;
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};

#[derive(Clone)]
struct DownloadRecord {
    url: String,
    folder: PathBuf,
    status: String,
}

struct VideoDownloaderApp {
    url: String,
    download_dir: Option<PathBuf>,
    status: String,
    progress_text: String,
    is_downloading: bool,
    tx: mpsc::Sender<String>,
    rx: mpsc::Receiver<String>,
    history: Vec<DownloadRecord>,
}

impl Default for VideoDownloaderApp {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel(100);
        let default_dir = dirs::download_dir();
        Self {
            url: String::new(),
            download_dir: default_dir,
            status: "Siap mengunduh.".to_string(),
            progress_text: "".to_string(),
            is_downloading: false,
            tx,
            rx,
            history: Vec::new(),
        }
    }
}

impl eframe::App for VideoDownloaderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle incoming messages from the background thread
        while let Ok(msg) = self.rx.try_recv() {
            if msg.starts_with("PROGRESS:") {
                self.progress_text = msg.replace("PROGRESS:", "").trim().to_string();
            } else {
                self.status = msg.clone();
                if msg == "Unduhan selesai!" || msg.starts_with("Error:") {
                    self.is_downloading = false;
                    self.progress_text.clear();
                    
                    if msg == "Unduhan selesai!" {
                        if let Some(dir) = &self.download_dir {
                            self.history.push(DownloadRecord {
                                url: self.url.clone(),
                                folder: dir.clone(),
                                status: "Berhasil".to_string(),
                            });
                        }
                        self.url.clear(); // Clear input
                    }
                }
            }
        }

        // Apply a clean, readable style
        let mut style = (*ctx.style()).clone();
        style.text_styles.get_mut(&egui::TextStyle::Body).unwrap().size = 15.0;
        style.text_styles.get_mut(&egui::TextStyle::Button).unwrap().size = 16.0;
        style.text_styles.get_mut(&egui::TextStyle::Heading).unwrap().size = 22.0;
        ctx.set_style(style);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🎬 Aplikasi Pengunduh Video");
            ui.add_space(10.0);

            // INPUT SECTION
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.label("🔗 Masukkan URL Video (misal: YouTube, TikTok, X):");
                let url_input = ui.add_sized([ui.available_width(), 32.0], egui::TextEdit::singleline(&mut self.url).hint_text("https://..."));
                
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label("📂 Lokasi Unduhan: ");
                    let dir_str = match &self.download_dir {
                        Some(path) => path.to_string_lossy().to_string(),
                        None => "Pilih Folder...".to_string(),
                    };
                    
                    if ui.button("📁 Ubah...").clicked() && !self.is_downloading {
                        if let Some(path) = FileDialog::new().pick_folder() {
                            self.download_dir = Some(path);
                        }
                    }
                    
                    if let Some(path) = &self.download_dir {
                        if ui.button("Buka Folder").clicked() {
                            let _ = open::that(path);
                        }
                    }
                    ui.label(dir_str);
                });

                ui.add_space(15.0);

                if self.is_downloading {
                    ui.horizontal(|ui| {
                        ui.add(egui::Spinner::new());
                        ui.label("Sedang mengunduh...");
                    });
                    if !self.progress_text.is_empty() {
                        ui.label(egui::RichText::new(&self.progress_text).monospace().color(egui::Color32::LIGHT_BLUE));
                    }
                } else {
                    if ui.button("▶️ Unduh Video").clicked() {
                        if self.url.is_empty() {
                            self.status = "Error: URL tidak boleh kosong!".to_string();
                        } else if self.download_dir.is_none() {
                            self.status = "Error: Lokasi unduhan belum dipilih!".to_string();
                        } else {
                            self.is_downloading = true;
                            self.status = "Memulai proses unduhan...".to_string();
                            self.progress_text.clear();
                            
                            let url = self.url.clone();
                            let download_dir = self.download_dir.clone().unwrap();
                            let tx = self.tx.clone();
                            let ctx_clone = ctx.clone();

                            // Spawn download process
                            thread::spawn(move || {
                                let _ = tx.blocking_send("Mencari info & mengunduh...".to_string());
                                ctx_clone.request_repaint();

                                // We use yt-dlp binary directly to parse stdout
                                let mut child = match Command::new("yt-dlp")
                                    .arg(&url)
                                    .arg("-P")
                                    .arg(&download_dir)
                                    .stdout(Stdio::piped())
                                    .stderr(Stdio::piped())
                                    .spawn() {
                                        Ok(c) => c,
                                        Err(e) => {
                                            let _ = tx.blocking_send(format!("Error: Gagal menjalankan yt-dlp (pastikan yt-dlp sudah terinstall). Detail: {}", e));
                                            ctx_clone.request_repaint();
                                            return;
                                        }
                                    };

                                if let Some(stdout) = child.stdout.take() {
                                    let reader = BufReader::new(stdout);
                                    for line in reader.lines() {
                                        if let Ok(l) = line {
                                            if l.contains("[download]") {
                                                let _ = tx.blocking_send(format!("PROGRESS:{}", l));
                                                ctx_clone.request_repaint();
                                            }
                                        }
                                    }
                                }

                                match child.wait() {
                                    Ok(status) if status.success() => {
                                        let _ = tx.blocking_send("Unduhan selesai!".to_string());
                                    }
                                    Ok(status) => {
                                        let _ = tx.blocking_send(format!("Error: Unduhan gagal dengan kode {:?}", status.code()));
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
                
                // Status message at the bottom of the input form
                ui.add_space(5.0);
                if self.status.starts_with("Error") {
                    ui.colored_label(egui::Color32::from_rgb(255, 100, 100), &self.status);
                } else if self.status == "Unduhan selesai!" {
                    ui.colored_label(egui::Color32::from_rgb(100, 255, 100), &self.status);
                } else {
                    ui.label(&self.status);
                }
            });

            ui.add_space(15.0);
            ui.separator();
            ui.add_space(10.0);

            // HISTORY SECTION
            ui.heading("📝 Riwayat Unduhan");
            ui.add_space(5.0);
            
            if self.history.is_empty() {
                ui.label(egui::RichText::new("Belum ada riwayat unduhan.").italics());
            } else {
                egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                    for record in self.history.iter().rev() {
                        egui::Frame::none()
                            .inner_margin(8.0)
                            .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(100)))
                            .rounding(4.0)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label("✅");
                                    ui.vertical(|ui| {
                                        ui.label(egui::RichText::new(&record.url).strong());
                                        if ui.button("Buka File / Folder Hasil Unduhan").clicked() {
                                            let _ = open::that(&record.folder);
                                        }
                                    });
                                });
                            });
                        ui.add_space(5.0);
                    }
                });
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
        .with_inner_size([700.0, 500.0])
        .with_min_inner_size([500.0, 400.0]);
        
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
