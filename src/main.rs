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
    pid: Option<u32>,
    is_paused: bool,
    progress_percent: f32,
    browser_cookie: String,
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
            pid: None,
            is_paused: false,
            progress_percent: 0.0,
            browser_cookie: "Tidak Pakai".to_string(),
        }
    }
}

impl eframe::App for VideoDownloaderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle incoming messages from the background thread
        while let Ok(msg) = self.rx.try_recv() {
            if msg.starts_with("PID:") {
                if let Ok(pid) = msg.replace("PID:", "").trim().parse::<u32>() {
                    self.pid = Some(pid);
                }
            } else if msg.starts_with("PROGRESS:") {
                let progress_str = msg.replace("PROGRESS:", "").trim().to_string();
                self.progress_text = progress_str.clone();
                if let Some(percent_idx) = progress_str.find("%") {
                    let text_before = &progress_str[..percent_idx];
                    let parts: Vec<&str> = text_before.split_whitespace().collect();
                    if let Some(last) = parts.last() {
                        if let Ok(p) = last.parse::<f32>() {
                            self.progress_percent = p / 100.0;
                        }
                    }
                }
            } else {
                self.status = msg.clone();
                if msg == "Unduhan selesai!" || msg.starts_with("Error:") {
                    self.is_downloading = false;
                    self.progress_text.clear();
                    self.pid = None;
                    self.progress_percent = 0.0;
                    self.is_paused = false;
                    
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

                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label("🍪 Akses Login (Browser):");
                    egui::ComboBox::from_id_source("browser_cookie_combo")
                        .selected_text(match self.browser_cookie.as_str() {
                            "chrome" => "Google Chrome",
                            "firefox" => "Mozilla Firefox",
                            "edge" => "Microsoft Edge",
                            "brave" => "Brave Browser",
                            "safari" => "Safari (Mac)",
                            "opera" => "Opera",
                            _ => "Tidak Pakai",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.browser_cookie, "Tidak Pakai".to_string(), "Tidak Pakai");
                            ui.selectable_value(&mut self.browser_cookie, "chrome".to_string(), "Google Chrome");
                            ui.selectable_value(&mut self.browser_cookie, "firefox".to_string(), "Mozilla Firefox");
                            ui.selectable_value(&mut self.browser_cookie, "edge".to_string(), "Microsoft Edge");
                            ui.selectable_value(&mut self.browser_cookie, "brave".to_string(), "Brave Browser");
                            ui.selectable_value(&mut self.browser_cookie, "safari".to_string(), "Safari (Mac)");
                            ui.selectable_value(&mut self.browser_cookie, "opera".to_string(), "Opera");
                        });
                });

                ui.add_space(15.0);

                if self.is_downloading {
                    ui.horizontal(|ui| {
                        if self.is_paused {
                            ui.label("⏸️ Dihentikan sementara...");
                        } else {
                            ui.add(egui::Spinner::new());
                            ui.label("⚡ Sedang mengunduh...");
                        }
                    });

                    if self.progress_percent > 0.0 {
                        ui.add_space(5.0);
                        let progress_bar = egui::ProgressBar::new(self.progress_percent)
                            .show_percentage()
                            .animate(!self.is_paused);
                        ui.add(progress_bar);
                    }

                    if !self.progress_text.is_empty() {
                        ui.add_space(5.0);
                        ui.label(egui::RichText::new(&self.progress_text).monospace().color(egui::Color32::LIGHT_BLUE));
                    }

                    if let Some(pid) = self.pid {
                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            if self.is_paused {
                                if ui.button("▶️ Lanjutkan").clicked() {
                                    #[cfg(unix)]
                                    let _ = Command::new("kill").arg("-CONT").arg(pid.to_string()).status();
                                    self.is_paused = false;
                                }
                            } else {
                                if ui.button("⏸️ Jeda").clicked() {
                                    #[cfg(unix)]
                                    let _ = Command::new("kill").arg("-STOP").arg(pid.to_string()).status();
                                    self.is_paused = true;
                                }
                            }

                            if ui.button("⏹️ Batal").clicked() {
                                #[cfg(unix)]
                                let _ = Command::new("kill").arg("-9").arg(pid.to_string()).status();
                                #[cfg(windows)]
                                let _ = Command::new("taskkill").arg("/F").arg("/PID").arg(pid.to_string()).status();
                                
                                self.is_downloading = false;
                                self.status = "Unduhan dibatalkan.".to_string();
                                self.pid = None;
                                self.progress_percent = 0.0;
                                self.is_paused = false;
                            }
                        });
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
                            self.progress_percent = 0.0;
                            self.is_paused = false;
                            self.pid = None;
                            
                            let url = self.url.clone();
                            let download_dir = self.download_dir.clone().unwrap();
                            let browser_cookie = self.browser_cookie.clone();
                            let tx = self.tx.clone();
                            let ctx_clone = ctx.clone();

                            // Spawn download process
                            thread::spawn(move || {
                                let _ = tx.blocking_send("Mencari info & mengunduh...".to_string());
                                ctx_clone.request_repaint();

                                // Construct yt-dlp command
                                let mut cmd = Command::new("yt-dlp");
                                cmd.arg(&url)
                                   .arg("-P")
                                   .arg(&download_dir);

                                if browser_cookie != "Tidak Pakai" {
                                    cmd.arg("--cookies-from-browser").arg(&browser_cookie);
                                }

                                let mut child = match cmd
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

                                let _ = tx.blocking_send(format!("PID:{}", child.id()));

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
                                        let _ = tx.blocking_send(format!("Error: Unduhan gagal atau dihentikan. (Kode: {:?})", status.code()));
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
