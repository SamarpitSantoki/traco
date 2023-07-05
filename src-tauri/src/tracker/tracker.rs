use serde::{Deserialize, Serialize};
use winapi::um::winuser::{GetForegroundWindow, GetWindowTextW};
use std::io::{Read, Write};
use std::sync::Mutex;
use std::time::SystemTime;
use std::{collections::HashMap,};
use tauri::State;
use tauri::{Manager, Window};
use uuid::Uuid;
use std::fs::File;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Item {
    name: String,
    start_time: u64,
    duration: u64,
}


pub struct AppState {
    pub keep_running: Mutex<bool>,
}


#[tauri::command]
pub fn start_tracking(window: Window) -> String {
    // spawn a new process to run the tracking app and return pid
    std::thread::spawn(move || {
        init_tracking(window);
    });

    format!("Tracking started")
}

// helper function to start tracking
pub fn init_tracking(window: tauri::Window) {
    let state: State<AppState> = window.state();
    let mut keep_running = state.keep_running.lock().unwrap();
    
    if *keep_running {
        println!("Tracking already started");
        return;
    }
    
    *keep_running = true;



    let process_id = Uuid::new_v4();
    let mut info: HashMap<String, Item> = HashMap::new();

    // keep track of current window title

    let mut current_window_title = String::new();
    let mut timer = SystemTime::now();

    // save to Desktop
    let desktop = std::env::var("USERPROFILE").unwrap() + "\\OneDrive/Documents/Traco/\\";

    // check if folder exists
    if !std::path::Path::new(&desktop).exists() {
        std::fs::create_dir(&desktop).unwrap();
    }

    // read a json file

    let mut file = File::open(desktop.clone() + "data.json").unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();


    if data.len() > 0 {
        info = serde_json::from_str(&data).unwrap();
    }

    println!("Tracking Starting");

    println!("Tracking Started - {}",*keep_running );

    let run = keep_running.clone();
    drop(keep_running);

    while  run {
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
            current_window_title = window_title;
        }

        // every 5 seconds

        if timer.elapsed().unwrap().as_secs() > 5 {
            println!("Saving - {}", process_id);
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

        if state.keep_running.lock().unwrap().clone() == false {
            window
                .emit("stop_tracking", Some("Stopped".to_string()))
                .unwrap();
            break;
        }

    }

    // gracefully handle exit
}


#[tauri::command]
pub fn stop_tracking(window: Window) {
    let state: State<AppState> = window.state();

    let mut keep_running = state.keep_running.lock().unwrap();

    *keep_running = false;

    drop(keep_running);
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
