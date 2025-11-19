use tray_icon::{
    menu::{Menu, MenuItem, MenuEvent},
    TrayIconBuilder,
};
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread;

slint::include_modules!();

// Messages from GTK thread to Slint thread
enum TrayEvent {
    ShowWindow,
    Exit,
}

// Messages from Slint thread to GTK thread
enum SlintEvent {
    Quit,
}

fn main() -> Result<(), slint::PlatformError> {
    // Create communication channels
    let (tray_tx, tray_rx) = mpsc::channel::<TrayEvent>();
    let (slint_tx, slint_rx) = mpsc::channel::<SlintEvent>();
    
    // Start GTK tray thread
    let gtk_handle = create_tray_icon(tray_tx, slint_rx);
    
    // Start Slint UI thread
    let slint_handle = instantiate_ui(tray_rx, slint_tx.clone());
    
    // Set up cross-thread event handling
    setup_event_handling(slint_tx);
    
    println!("Application is running. Both event loops are active.");

    // Wait for threads to complete
    let slint_result = slint_handle.join().expect("Slint thread panicked");
    gtk_handle.join().expect("GTK thread panicked");

    println!("Application is closing.");

    slint_result
}

//#[cfg(target_os = "macos")]
fn create_tray_icon(
    tray_tx: Sender<TrayEvent>,
    slint_rx: Receiver<SlintEvent>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        // Initialize GTK in this thread
        gtk::init().expect("Failed to initialize GTK");
        
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tray_icon.png");
        let (dark_icon, light_icon) = load_and_invert_icon(path);

        let icon = match dark_light::detect() {
            Ok(dark_light::Mode::Dark) => light_icon,
            _ => dark_icon,
        };

        let tray_menu = Menu::new();
        let show_item = MenuItem::new("Show Window", true, None);
        let exit_item = MenuItem::new("Exit", true, None);
        
        let show_id = show_item.id().clone();
        let exit_id = exit_item.id().clone();
        
        tray_menu.append(&show_item).unwrap();
        tray_menu.append(&exit_item).unwrap();

        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip("Tray Test with Slint")
            .with_icon(icon)
            .build()
            .unwrap();

        println!("Tray icon has been set up in GTK thread.");

        // Keep tray icon alive
        let _tray_icon = tray_icon;

        // GTK event loop with menu event handling
        let menu_channel = MenuEvent::receiver();
        
        loop {
            // Process GTK events
            while gtk::events_pending() {
                gtk::main_iteration_do(false);
            }
            
            // Check for menu events
            while let Ok(event) = menu_channel.try_recv() {
                if event.id == show_id {
                    println!("Show menu item clicked in GTK thread");
                    tray_tx.send(TrayEvent::ShowWindow).ok();
                } else if event.id == exit_id {
                    println!("Exit menu item clicked in GTK thread");
                    tray_tx.send(TrayEvent::Exit).ok();
                    return; // Exit GTK thread
                }
            }
            
            // Check for messages from Slint thread
            if let Ok(SlintEvent::Quit) = slint_rx.try_recv() {
                println!("Quit signal received in GTK thread");
                return; // Exit GTK thread
            }
            
            // Small sleep to prevent busy-waiting
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    })
}



fn instantiate_ui(
    tray_rx: Receiver<TrayEvent>,
    slint_tx: Sender<SlintEvent>,
) -> thread::JoinHandle<Result<(), slint::PlatformError>> {
    thread::spawn(move || {
        let main_window = MainWindow::new().unwrap();

        println!("Main window has been instantiated in Slint thread.");

        let window = main_window.window();

        let weak_window = main_window.as_weak();
        window.on_close_requested(move || on_close_requested(weak_window.clone()));

        let slint_tx_clone = slint_tx.clone();
        main_window.on_quit(move || on_quit(slint_tx_clone.clone()));

        // Handle tray events in Slint thread using a timer
        let weak_ui = main_window.as_weak();
        let timer = slint::Timer::default();
        timer.start(
            slint::TimerMode::Repeated,
            std::time::Duration::from_millis(10),
            move || {
                while let Ok(event) = tray_rx.try_recv() {
                    match event {
                        TrayEvent::ShowWindow => {
                            println!("Show window event received in Slint thread");
                            if let Some(ui) = weak_ui.upgrade() {
                                ui.show().unwrap();
                            }
                        }
                        TrayEvent::Exit => {
                            println!("Exit event received in Slint thread");
                            slint::quit_event_loop().ok();
                        }
                    }
                }
            },
        );

        // Run Slint event loop
        slint::run_event_loop_until_quit()
    })
}

fn setup_event_handling(_slint_tx: Sender<SlintEvent>) {
    // All cross-thread communication is now handled within the thread functions
    // This function is kept for future extensibility
    println!("Event handling setup complete.");
}

fn on_close_requested(win: slint::Weak<MainWindow>) -> slint::CloseRequestResponse {
    if let Some(win) = win.upgrade() {
        win.hide().unwrap();
        println!("Main window has been hidden.");
    }
    slint::CloseRequestResponse::HideWindow
}

fn on_quit(slint_tx: Sender<SlintEvent>) {
    println!("Quit event received. Closing application.");
    slint_tx.send(SlintEvent::Quit).ok();
    slint::quit_event_loop().unwrap();
}

fn load_and_invert_icon(path: &str) -> (tray_icon::Icon, tray_icon::Icon) {
    let mut img = image::open(path)
        .expect("Failed to open icon path")
        .into_rgba8();

    let (width, height) = img.dimensions();
    let original_rgba = img.to_vec();
    let original_icon =
        tray_icon::Icon::from_rgba(original_rgba, width, height).expect("Failed to create original icon");

    image::imageops::invert(&mut img);

    let inverted_rgba = img.into_raw();
    let inverted_icon =
        tray_icon::Icon::from_rgba(inverted_rgba, width, height).expect("Failed to create inverted icon");

    (original_icon, inverted_icon)
}