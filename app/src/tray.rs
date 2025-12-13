use tray_icon::{
    menu::{Menu, MenuItem},
    Icon, TrayIcon, TrayIconBuilder,
};

use crate::{APP_ICON, DENY_HIDING};

pub const SHOW_ID: &str = "tray-show";
pub const QUIT_ID: &str = "tray-quit";

struct TrayMenuItems {
    #[allow(dead_code)]
    show: MenuItem,
    quit: MenuItem,
}

impl TrayMenuItems {
    fn build() -> Self {
        let show = MenuItem::with_id(SHOW_ID, "Show", true, None);
        let quit = MenuItem::with_id(QUIT_ID, "Quit", true, None);

        Self { show, quit }
    }
}

fn build_tray_menu(items: &TrayMenuItems, has_gui: bool) -> Menu {
    let menu = Menu::new();
    if has_gui && !*DENY_HIDING {
        menu.append_items(&[&items.show]).unwrap();
    }
    menu.append_items(&[&items.quit]).unwrap();
    menu
}

pub fn build_tray(has_gui: bool) -> Option<TrayIcon> {
    let items = TrayMenuItems::build();
    let menu = build_tray_menu(&items, has_gui);

    // 🛑 الرجاء ملاحظة: لقد أزلنا .with_menu_on_left_click(false) 
    // لتجنب تعقيدات سلوك النقر الافتراضي ونعتمد على المكتبة لإرسال حدث عند النقر الأيسر.
    TrayIconBuilder::new()
        .with_tooltip("Legion Keyboard Control")
        .with_icon(load_tray_icon(APP_ICON))
        .with_menu(Box::new(menu))
        .build()
        .ok()
}

#[must_use]
fn load_tray_icon(image_data: &[u8]) -> Icon {
    use tray_icon::Icon;

    let image = image::load_from_memory(image_data).unwrap();
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.into_flat_samples().samples;

    Icon::from_rgba(pixels, image.width(), image.height()).unwrap()
}
