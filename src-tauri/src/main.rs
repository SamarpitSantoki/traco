// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::time::SystemTime;
use std::{collections::HashMap, io::Read};
use tauri::{CustomMenuItem, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem};
use tauri::{Manager, SystemTray, Window};
use winapi::um::winuser::{GetForegroundWindow, GetWindowTextW};

#[derive(Serialize, Deserialize, Clone)]
struct Item {
    name: String,
    start_time: u64,
    duration: u64,
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn tracking(window: Window) -> String {
    // spawn a new process to run the tracking app

    std::thread::spawn(move || {
        app(window);
    });

    format!("Tracking started")
}
#[derive(Clone, serde::Serialize)]
struct Payload {
    args: Vec<String>,
    cwd: String,
}

fn main() {
    // let tray_menu = SystemTrayMenu::new(); // insert the menu items here

    // here `"quit".to_string()` defines the menu item id, and the second parameter is the menu item label.
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let hide = CustomMenuItem::new("hide".to_string(), "Hide");

    let tray_menu = SystemTrayMenu::new()
        .add_item(quit)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(hide);

    let tray = SystemTray::new().with_menu(tray_menu);

    tauri::Builder::default()
        .on_window_event(|event| match event.event() {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                event.window().hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            println!("{}, {argv:?}, {cwd}", app.package_info().name);

            app.emit_all("single-instance", Payload { args: argv, cwd })
                .unwrap();
        }))
        .system_tray(tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::LeftClick {
                position: _,
                size: _,
                ..
            } => {
                let window = app.get_window("main").unwrap();

                window.show().unwrap();

                println!("system tray received a left click");
            }
            SystemTrayEvent::RightClick {
                position: _,
                size: _,
                ..
            } => {
                println!("system tray received a right click");
            }
            SystemTrayEvent::DoubleClick {
                position: _,
                size: _,
                ..
            } => {
                println!("system tray received a double click");
            }
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "quit" => {
                    std::process::exit(0);
                }
                "hide" => {
                    let window = app.get_window("main").unwrap();
                    window.hide().unwrap();
                }
                _ => {}
            },
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![greet, tracking])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    // app();
}

fn app(window: tauri::Window) {
    let mut info: HashMap<String, Item> = HashMap::new();

    // keep track of current window title

    let mut current_window_title = String::new();
    let up_time = SystemTime::now();
    let mut timer = SystemTime::now();

    // save to Desktop
    let desktop = std::env::var("USERPROFILE").unwrap() + "\\OneDrive/Documents/Traco/\\";

    // check if folder exists
    if !std::path::Path::new(&desktop).exists() {
        std::fs::create_dir(&desktop).unwrap();
    }

    let mut file = File::open(desktop.clone() + "data.json")
        .unwrap_or(File::create(desktop.clone() + "data.json").unwrap());

    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    if data.len() > 0 {
        info = serde_json::from_str(&data).unwrap();
    }

    loop {
        // get current window title

        let window_title = get_window_title();

        if info.contains_key(&window_title) {
            let mut item = info.get_mut(&window_title).unwrap();
            item.duration += 1;
        } else {
            let item = Item {
                name: window_title.clone(),
                start_time: SystemTime::now()
                    .duration_since(
                        SystemTime::UNIX_EPOCH
                            .checked_add(std::time::Duration::from_secs(1))
                            .unwrap(),
                    )
                    .unwrap()
                    .as_secs(),
                duration: 1,
            };
            info.insert(window_title.clone(), item);
        }

        // if window title has changed
        if window_title != current_window_title {
            // print window title

            println!("{}", window_title);

            // update current window title

            current_window_title = window_title;
        }

        // every 5 seconds

        println!("{} seconds left", timer.elapsed().unwrap().as_secs());
        if timer.elapsed().unwrap().as_secs() > 5 {
            // let mut file = File::create("data.json").unwrap();
            let mut file = File::create(desktop.clone() + "data.json").unwrap();
            let json = serde_json::to_string(&info).unwrap();
            file.write_all(json.as_bytes()).unwrap();
            println!("Saved");
            timer = SystemTime::now();

            window
                .emit("tracking", Some("Tracking".to_string()))
                .unwrap();
        }

        // sleep for 1 second

        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    // gracefully handle exit
}

fn get_window_title() -> String {
    unsafe {
        let hwnd = GetForegroundWindow();

        let mut title = [0u16; 512];

        GetWindowTextW(hwnd, title.as_mut_ptr(), 512);

        let title = String::from_utf16_lossy(&title);

        title.to_string().replace("\u{0}", "")
    }
}
