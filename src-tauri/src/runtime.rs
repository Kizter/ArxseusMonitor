//! Bagian Tauri: tray, jendela, command IPC, dan penyambungan poller.
//!
//! Semua teks yang terlihat pengguna berasal dari frontend. Rust hanya
//! mengirim `level` dan `reasonKey`; label menu dan tooltip dikirim balik oleh
//! React lewat `configure_tray` / `set_tray_tooltip` saat bahasa berubah.

use crate::health::Level;
use crate::monitor::model::Snapshot;
use crate::monitor::poller::{self, MonitorHandle};
use crate::settings::{self, Settings};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, State, WebviewWindow, WindowEvent,
};

pub const TRAY_ID: &str = "main";
pub const WINDOW_LABEL: &str = "main";

const MENU_OPEN: &str = "open";
const MENU_SETTINGS: &str = "settings";
const MENU_QUIT: &str = "quit";

/// State yang dibagi antara poller, command, dan tray.
pub struct Runtime {
    settings: Mutex<Settings>,
    snapshot: Mutex<Option<Snapshot>>,
    monitor: Mutex<Option<MonitorHandle>>,
    /// Mode demo: paksa semua sumber suhu nonaktif (DESIGN.md §10 butir 5).
    /// Bisa dinyalakan belakangan lewat instance kedua yang membawa flag.
    simulate_missing_temp: AtomicBool,
    /// Level ikon tray terakhir, supaya ikon tidak digambar ulang tiap 2 detik.
    last_level: AtomicU8,
}

impl Runtime {
    pub fn new(settings: Settings, simulate_missing_temp: bool) -> Self {
        Self {
            settings: Mutex::new(settings),
            snapshot: Mutex::new(None),
            monitor: Mutex::new(None),
            simulate_missing_temp: AtomicBool::new(simulate_missing_temp),
            last_level: AtomicU8::new(UNSET_LEVEL),
        }
    }
}

const UNSET_LEVEL: u8 = u8::MAX;

fn level_code(level: Level) -> u8 {
    match level {
        Level::Unknown => 0,
        Level::Safe => 1,
        Level::Warn => 2,
        Level::Danger => 3,
    }
}

/// Label menu tray. Dikirim frontend agar tetap bilingual.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrayLabels {
    pub open: String,
    pub settings: String,
    pub quit: String,
}

// ---------------------------------------------------------------- tray icon

/// Ikon tray digambar saat runtime: lingkaran penuh berwarna status. Ukuran 32
/// piksel, di-downscale Windows sendiri.
pub fn tray_icon(level: Level) -> Image<'static> {
    const SIZE: u32 = 32;
    const RADIUS: f32 = 14.0;

    let (r, g, b) = match level {
        Level::Safe => (34, 197, 94),
        Level::Warn => (245, 158, 11),
        Level::Danger => (239, 68, 68),
        Level::Unknown => (148, 163, 184),
    };

    let center = (SIZE as f32 - 1.0) / 2.0;
    let mut rgba = vec![0u8; (SIZE * SIZE * 4) as usize];

    for y in 0..SIZE {
        for x in 0..SIZE {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let distance = (dx * dx + dy * dy).sqrt();
            // Tepi dibuat lembut setengah piksel agar tidak bergerigi.
            let alpha = ((RADIUS - distance + 0.5).clamp(0.0, 1.0) * 255.0).round() as u8;

            let index = ((y * SIZE + x) * 4) as usize;
            rgba[index] = r;
            rgba[index + 1] = g;
            rgba[index + 2] = b;
            rgba[index + 3] = alpha;
        }
    }

    Image::new_owned(rgba, SIZE, SIZE)
}

fn update_tray(app: &AppHandle, snapshot: &Snapshot) {
    let runtime = app.state::<Runtime>();
    let code = level_code(snapshot.overall);

    if runtime.last_level.swap(code, Ordering::Relaxed) == code {
        return;
    }

    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_icon(Some(tray_icon(snapshot.overall)));
    }
}

// ------------------------------------------------------------ window helper

/// Tampilkan jendela di pojok kanan bawah area kerja, seperti menempel pada
/// tray. Perhitungan dilakukan manual supaya tidak perlu plugin tambahan.
pub fn show_window(app: &AppHandle) {
    let Some(window) = app.get_webview_window(WINDOW_LABEL) else {
        return;
    };

    place_window(&window);
    let _ = window.show();
    let _ = window.unminimize();
    let _ = window.set_focus();
}

fn place_window(window: &WebviewWindow) {
    let Ok(Some(monitor)) = window.current_monitor() else {
        return;
    };

    let area = monitor.size();
    let origin = monitor.position();
    let scale = window.scale_factor().unwrap_or(1.0);
    let size = window.outer_size().unwrap_or_else(|_| {
        PhysicalSize::new((360.0 * scale) as u32, (520.0 * scale) as u32)
    });

    // 12 px dari tepi kanan, 56 px dari bawah (perkiraan taskbar Windows).
    let margin = (12.0 * scale) as i32;
    let taskbar = (56.0 * scale) as i32;

    let x = origin.x + area.width as i32 - size.width as i32 - margin;
    let y = origin.y + area.height as i32 - size.height as i32 - taskbar;

    let _ = window.set_position(PhysicalPosition::new(x.max(origin.x), y.max(origin.y)));
}

fn toggle_window(app: &AppHandle) {
    let Some(window) = app.get_webview_window(WINDOW_LABEL) else {
        return;
    };

    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
    } else {
        show_window(app);
    }
}

// ------------------------------------------------------------------- wiring

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    TrayIconBuilder::with_id(TRAY_ID)
        .icon(tray_icon(Level::Unknown))
        .tooltip("ArxseusMonitor")
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                toggle_window(tray.app_handle());
            }
        })
        .on_menu_event(|app, event| match event.id().as_ref() {
            MENU_OPEN => show_window(app),
            MENU_SETTINGS => {
                show_window(app);
                let _ = app.emit("open-settings", ());
            }
            MENU_QUIT => {
                println!("[app] keluar dari menu tray");
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    Ok(())
}

fn spawn_monitor(app: &AppHandle, settings: Settings, simulate_missing_temp: bool) -> MonitorHandle {
    let handle = app.clone();

    poller::spawn(settings, simulate_missing_temp, move |snapshot| {
        let runtime = handle.state::<Runtime>();
        if let Ok(mut cache) = runtime.snapshot.lock() {
            *cache = Some(snapshot.clone());
        }

        update_tray(&handle, &snapshot);
        let _ = handle.emit("snapshot", &snapshot);
    })
}

fn restart_monitor(app: &AppHandle, settings: &Settings) {
    let runtime = app.state::<Runtime>();

    if let Ok(mut slot) = runtime.monitor.lock()
        && let Some(handle) = slot.take()
    {
        // `stop()` menunggu thread selesai; jangan lakukan di UI thread.
        std::thread::spawn(move || drop(handle));
    }

    let handle = spawn_monitor(
        app,
        settings.clone(),
        runtime.simulate_missing_temp.load(Ordering::SeqCst),
    );
    if let Ok(mut slot) = runtime.monitor.lock() {
        *slot = Some(handle);
    }
}

/// Dipanggil `tauri-plugin-single-instance` saat instance kedua dijalankan.
///
/// Argumen instance kedua ikut dibaca: `--simulate-missing-temp` harus berlaku
/// juga saat aplikasi sudah lama hidup di tray (mis. jendela demo sidang lupa
/// ditutup lebih dulu). Tanpa ini flag tersebut diam-diam tidak berefek.
pub fn on_second_instance(app: &AppHandle, args: &[String]) {
    println!("[app] instance kedua terdeteksi: tampilkan jendela yang ada");

    let runtime = app.state::<Runtime>();
    let wants_simulation = args.iter().any(|arg| arg == "--simulate-missing-temp");

    if wants_simulation && !runtime.simulate_missing_temp.swap(true, Ordering::SeqCst) {
        println!("[app] instance kedua meminta mode simulasi: semua sumber suhu dimatikan");

        let settings = runtime
            .settings
            .lock()
            .map(|settings| settings.clone())
            .unwrap_or_default();
        restart_monitor(app, &settings);
    }

    show_window(app);
}

/// Dipanggil dari `setup` Tauri.
pub fn init(app: &AppHandle, simulate_missing_temp: bool) -> tauri::Result<()> {
    let settings = settings::load(&settings::default_path());
    if simulate_missing_temp {
        println!("[app] mode simulasi: semua sumber suhu dimatikan");
    }
    app.manage(Runtime::new(settings.clone(), simulate_missing_temp));

    build_tray(app)?;
    restart_monitor(app, &settings);

    if !settings.start_minimized {
        show_window(app);
    }

    Ok(())
}

// ----------------------------------------------------------------- commands

#[tauri::command]
pub fn get_snapshot(runtime: State<'_, Runtime>) -> Option<Snapshot> {
    runtime
        .snapshot
        .lock()
        .ok()
        .and_then(|snapshot| snapshot.clone())
}

#[tauri::command]
pub fn get_settings(runtime: State<'_, Runtime>) -> Settings {
    runtime
        .settings
        .lock()
        .map(|settings| settings.clone())
        .unwrap_or_default()
}

#[tauri::command]
pub fn save_settings(
    app: AppHandle,
    runtime: State<'_, Runtime>,
    settings: Settings,
) -> Result<Settings, String> {
    let next = settings.sanitized();

    settings::save(&settings::default_path(), &next).map_err(|error| error.to_string())?;

    let previous = runtime
        .settings
        .lock()
        .map(|current| current.clone())
        .unwrap_or_default();

    if let Ok(mut current) = runtime.settings.lock() {
        *current = next.clone();
    }

    let needs_restart = previous.interval_secs != next.interval_secs
        || previous.temperature_source != next.temperature_source;

    if needs_restart {
        restart_monitor(&app, &next);
    }

    Ok(next)
}

/// Bangun ulang menu tray dengan label sesuai bahasa aktif.
#[tauri::command]
pub fn configure_tray(app: AppHandle, labels: TrayLabels) -> Result<(), String> {
    let open = MenuItem::with_id(&app, MENU_OPEN, &labels.open, true, None::<&str>)
        .map_err(|error| error.to_string())?;
    let settings_item = MenuItem::with_id(&app, MENU_SETTINGS, &labels.settings, true, None::<&str>)
        .map_err(|error| error.to_string())?;
    let quit =
        MenuItem::with_id(&app, MENU_QUIT, &labels.quit, true, None::<&str>)
            .map_err(|error| error.to_string())?;

    let menu = Menu::with_items(&app, &[&open, &settings_item, &quit])
        .map_err(|error| error.to_string())?;

    let tray = app
        .tray_by_id(TRAY_ID)
        .ok_or_else(|| "ikon tray tidak ditemukan".to_string())?;

    tray.set_menu(Some(menu)).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn set_tray_tooltip(app: AppHandle, text: String) -> Result<(), String> {
    let tray = app
        .tray_by_id(TRAY_ID)
        .ok_or_else(|| "ikon tray tidak ditemukan".to_string())?;

    tray.set_tooltip(Some(text))
        .map_err(|error| error.to_string())
}

// --------------------------------------------------------------- lifecycle

/// Pasang perilaku jendela: tutup = sembunyi ke tray, bukan keluar.
pub fn handle_window_event(window: &tauri::Window, event: &WindowEvent) {
    if let WindowEvent::CloseRequested { api, .. } = event {
        api.prevent_close();
        let _ = window.hide();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pixel(image: &Image<'_>, x: u32, y: u32) -> [u8; 4] {
        let index = ((y * image.width() + x) * 4) as usize;
        let rgba = image.rgba();
        [rgba[index], rgba[index + 1], rgba[index + 2], rgba[index + 3]]
    }

    #[test]
    fn tray_icon_is_a_filled_circle() {
        let image = tray_icon(Level::Safe);

        assert_eq!(image.width(), 32);
        assert_eq!(image.height(), 32);
        assert_eq!(image.rgba().len(), 32 * 32 * 4);

        let center = pixel(&image, 16, 16);
        assert_eq!(&center[..3], &[34, 197, 94]);
        assert_eq!(center[3], 255);

        // Sudut tetap transparan.
        assert_eq!(pixel(&image, 0, 0)[3], 0);
    }

    #[test]
    fn each_level_has_its_own_colour() {
        let colours = [
            pixel(&tray_icon(Level::Safe), 16, 16)[..3].to_vec(),
            pixel(&tray_icon(Level::Warn), 16, 16)[..3].to_vec(),
            pixel(&tray_icon(Level::Danger), 16, 16)[..3].to_vec(),
            pixel(&tray_icon(Level::Unknown), 16, 16)[..3].to_vec(),
        ];

        for (index, colour) in colours.iter().enumerate() {
            assert!(
                !colours[..index].contains(colour),
                "warna level {index} duplikat"
            );
        }
    }
}
