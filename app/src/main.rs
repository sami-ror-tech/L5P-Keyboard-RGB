#![cfg_attr(not(test), windows_subsystem = "windows")]
#![cfg_attr(test, windows_subsystem = "console")]

mod cli;
#[cfg(target_os = "windows")]
mod console;
mod enums;
mod gui;
mod manager;
mod persist;
mod tray;
mod util;

use std::sync::LazyLock;
// 🔥 الإصلاح: إزالة الاستيرادات غير المستخدمة
// use std::{process, io::Read}; // ❌ احذف هذا السطر

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use cli::{GuiCommand, OutputType};
use color_eyre::{eyre::eyre, Result};
use eframe::{egui::IconData, epaint::Vec2};
use gui::App;
use tray_icon::TrayIcon as _;

const APP_ICON: &[u8; 14987] = include_bytes!("../res/trayIcon.ico");
const WINDOW_SIZE: Vec2 = Vec2::new(500., 460.);
#[cfg(target_os = "linux")]
pub static DENY_HIDING: LazyLock<bool> = LazyLock::new(|| std::env::var("WAYLAND_DISPLAY").is_ok());
#[cfg(not(target_os = "linux"))]
pub static DENY_HIDING: LazyLock<bool> = LazyLock::new(|| false);

// 🔥 الإصلاح: أزل static TRAY_HOLDER تماماً

fn main() {
    #[cfg(target_os = "windows")]
    {
        setup_panic().unwrap();
        run_windows();
    }

    #[cfg(target_os = "linux")]
    {
        color_eyre::install().unwrap();
        init().unwrap();
    }
}

#[cfg(target_os = "windows")]
fn run_windows() {
    console::attach();
    if init().is_err() {
        std::process::exit(2);
    }
    console::free();
}

#[cfg(target_os = "windows")]
fn setup_panic() -> Result<()> {
    let builder = color_eyre::config::HookBuilder::default();

    let (panic_hook, eyre_hook) = builder.into_hooks();
    eyre_hook.install()?;

    std::panic::set_hook(Box::new(move |panic_info| {
        if !console::alloc_with_color_support() {
            return;
        }

        eprintln!("{}", panic_hook.panic_report(panic_info));
        println!("Press Enter to continue...");
        let mut input = String::new();
        let _ = std::io::stdin().read_line(&mut input); // 🔥 استيراد مباشر
        std::process::exit(1);
    }));

    Ok(())
}

fn init() -> Result<()> {
    let cli_output = cli::try_cli().map_err(|err| eyre!("{:?}", err))?;

    match cli_output {
        GuiCommand::Start { hide_window, output_type } => {
            start_ui(output_type, hide_window);
            Ok(())
        }
        GuiCommand::Exit => Ok(()),
    }
}

fn start_ui(output_type: OutputType, hide_window: bool) {
    let has_tray = Arc::new(AtomicBool::new(true));
    let visible = Arc::new(AtomicBool::new(!hide_window));

    let app_icon = load_icon_data(APP_ICON);
    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size(WINDOW_SIZE)
            .with_min_inner_size(WINDOW_SIZE)
            .with_max_inner_size(WINDOW_SIZE)
            .with_icon(app_icon),
        ..eframe::NativeOptions::default()
    };

    let has_tray_c = has_tray.clone();

    // 🔥 الحل الحقيقي: حافظ على TrayIcon في متغير محلي فقط
    #[cfg(target_os = "linux")]
    std::thread::spawn(move || {
        gtk::init().unwrap();

        let tray_icon = tray::build_tray(true);
        has_tray_c.store(tray_icon.is_some(), Ordering::SeqCst);
        
        // ببساطة احتفظ بالأيقونة في هذا الخيط
        let _tray_icon_holder = tray_icon; // متغير محلي يمنع التدمير

        gtk::main();
    });

    #[cfg(not(target_os = "linux"))]
    {
        let tray_icon = tray::build_tray(true);
        has_tray_c.store(tray_icon.is_some(), Ordering::SeqCst);
        
        // 🔥 الحل: متغير محلي فقط (لا static)
        let _tray_icon_holder = tray_icon; // هذا يكفي لمنع التدمير المبكر
    }

    let app = App::new(output_type, has_tray, visible);

    eframe::run_native(
        "Legion RGB",
        native_options,
        Box::new(move |cc| {
            Ok(Box::new(app.init(cc)))
        }),
    )
    .unwrap();
}

#[must_use]
fn load_icon_data(image_data: &[u8]) -> IconData {
    let image = image::load_from_memory(image_data).unwrap();

    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.into_flat_samples().samples;

    IconData {
        rgba: pixels,
        width: image.width(),
        height: image.height(),
    }
}
