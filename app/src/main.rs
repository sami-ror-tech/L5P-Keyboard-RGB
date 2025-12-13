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

    // Since egui uses winit under the hood and doesn't use gtk on Linux, and we need gtk for
    // the tray icon to show up, we need to spawn a thread
    // where we initialize gtk and create the tray_icon
    #[cfg(target_os = "linux")]
    std::thread::spawn(move || {
        gtk::init().unwrap();

        let tray_icon = tray::build_tray(true);
        has_tray_c.store(tray_icon.is_some(), Ordering::SeqCst);

        gtk::main();
    });

    // 💡 التعديل الحاسم: بناء أيقونة الـ Tray وتخزينها هنا في متغير محلي (`_tray_icon`)
    // لضمان بقائها حية (Alive) طوال فترة تشغيل eframe::run_native.
    #[cfg(not(target_os = "linux"))]
    let _tray_icon = tray::build_tray(true);

    #[cfg(not(target_os = "linux"))]
    has_tray_c.store(_tray_icon.is_some(), Ordering::SeqCst);
    // تم إزالة المنطق السابق المعقد وغير الفعال للتعامل مع الـ TrayIcon داخل الـ Closure

    let app = App::new(output_type, has_tray, visible);

    eframe::run_native(
        "Legion RGB",
        native_options,
        // الـ Closure لم يعد يحتاج لبناء الـ Tray
        Box::new(move |cc| {
            Ok(Box::new(app.init(cc)))
        }),
    )
    .unwrap();
    // ينتهي نطاق _tray_icon هنا، وعندما يتم إنهاء run_native، سيتم التخلص منه
}

fn load_icon_data(image_data: &[u8]) -> IconData {
    // ... (بقية الدالة كما هي)
