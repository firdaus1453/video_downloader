# 🎬 Pengunduh Video Sederhana

Aplikasi ringan untuk mengunduh video dari YouTube dan berbagai platform lainnya, dibuat menggunakan Rust. Sangat mudah digunakan!

## 📥 Cara Mengunduh (Untuk Pengguna Biasa)

Anda tidak perlu menginstal kode yang rumit! Cukup ikuti langkah berikut:

1. Pergi ke halaman **[Releases](https://github.com/firdaus1453/video_downloader/releases)** di sebelah kanan halaman ini.
2. Unduh file aplikasi yang sesuai dengan sistem operasi Anda:
   - **Windows**: Unduh file berakhiran `.exe` atau khusus versi Windows.
   - **Mac (Apple)**: Unduh file khusus MacOS.
   - **Linux**: Unduh file Linux.
3. Jalankan aplikasi yang sudah diunduh seperti biasa!

**Penting:** Aplikasi ini memerlukan `yt-dlp` untuk bekerja dengan maksimal. Pastikan komputer Anda sudah terpasang `yt-dlp`.
- **Pengguna Mac**: Buka `Terminal` dan ketik `brew install yt-dlp` lalu Enter.
- **Pengguna Windows**: Disarankan mengunduh `yt-dlp.exe` dari [Situs Resmi yt-dlp](https://github.com/yt-dlp/yt-dlp/releases) dan menaruh file tersebut di dalam folder yang sama dengan Pengunduh Video ini.

*(Tips: Saat mac memperingatkan tentang Gatekeeper ("Aplikasi ini dari developer yang tidak dikenal"), Anda dapat membukanya dengan klik kanan -> **Open** (Buka) atau izinkan melalui System Settings (Pengaturan Sistem) > Privacy & Security.)*

---

## 💻 Panduan Untuk Pengembang (Developer)

Jika Anda ingin memodifikasi atau membangunnya dari awal (build from source), Anda memerlukan [Rust](https://rustup.rs/) terpasang di komputer Anda.

1. Buka folder ini di Terminal / Command Prompt.
2. Jalankan perintah kompilasi:
   ```bash
   cargo build --release
   ```
3. File utama (*executable*) akan tersedia di folder `target/release/video_downloader`.

Semoga bermanfaat!
