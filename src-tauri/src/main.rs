// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use core_graphics::{
    event::CGEvent,
    event_source::{CGEventSource, CGEventSourceStateID}
};
use tauri::{LogicalPosition, LogicalSize, Manager, Size};
use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial, NSVisualEffectState};
use std::time::{Duration, Instant};
use tauri_plugin_positioner::WindowExt;

// tauri.conf.jsonを確認
const HEIGHT: f64 = 800.0;
const WIDTH: f64 = 180.0;
const VELOCITY: f64 = WIDTH / 5.0;
const MAX_TOUCH_EDGE_COUNT: u32 = 15;
const UPDATE_INTERVAL: Duration = Duration::from_millis(1000 / 60);


// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .setup(|app| {
            let mut state = AppState::new();

            let main_window = app.get_webview_window("main").unwrap();

            #[cfg(target_os = "macos")]
            apply_vibrancy(
                &main_window,
                NSVisualEffectMaterial::HudWindow,
                Some(NSVisualEffectState::Active),
                Some(16.0)
                ).expect("Unsupported platform! 'apply_vibrancy' is only supported on macOS"
            );

            std::thread::spawn(move || {

                // window setting
                main_window.set_size(Size::Logical(
                        LogicalSize::new(WIDTH, HEIGHT)
                )).unwrap();
                main_window.move_window(tauri_plugin_positioner::Position::LeftCenter).unwrap();
             
                std::thread::sleep(Duration::from_secs(1));


                // observe mouse
                loop {
                    state.update();
                    state.slide_window(&main_window);
                    state.wait_next_update();
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}



#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Coords {
    pub x: f64,
    pub y: f64
}

impl Coords {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

pub struct Mouse;

impl Mouse {
    pub fn new() -> Self {
        Self
    }

    pub fn get_coords(&mut self) -> Option<Coords> {

        let event = CGEvent::new(
           CGEventSource::new(
               CGEventSourceStateID::CombinedSessionState
           ).unwrap()
       );

        let current_coords = match event {
            Ok(event) => {
                let point = event.location();
                Some(Coords::new(point.x, point.y))
            },
            Err(_) => None
        };

        current_coords
    }
}

pub struct AppState {
    mouse: Mouse,
    mouse_coords: Option<Coords>,
    time_instant: Instant,
    touch_edge_count: u32,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            mouse: Mouse::new(),
            mouse_coords: None,
            time_instant: Instant::now(),
            touch_edge_count: 0,
        }
    }

    pub fn update(&mut self) {
        self.time_instant = Instant::now();
        self.mouse_coords = self.mouse.get_coords();
        if let Some(coords) = self.mouse_coords {
            if coords.x == 0.0 {
                self.touch_edge_count += 1;
            } else {
                self.touch_edge_count = 0;
            }
        };
    }

    pub fn wait_next_update(&mut self) {
        let elapsed = self.time_instant.elapsed();
        if elapsed < UPDATE_INTERVAL {
            std::thread::sleep(UPDATE_INTERVAL - elapsed);
        }
    }

    pub fn required_to_show_window(&mut self, main_window: &tauri::WebviewWindow) -> bool {
        let window_position = main_window
            .outer_position()
            .unwrap()
            .to_logical::<f64>(main_window.scale_factor().unwrap());

        if let Some(coords) = self.mouse_coords {
            if coords.x == 0.0
                && self.touch_edge_count >= MAX_TOUCH_EDGE_COUNT
                && coords.x <= window_position.x + WIDTH
                && window_position.x < 0.0 {
                return true;
            }
        };

        false
    }

    pub fn required_to_hide_window(&mut self, main_window: &tauri::WebviewWindow) -> bool {
        let window_position = main_window
            .outer_position()
            .unwrap()
            .to_logical::<f64>(main_window.scale_factor().unwrap());

        if let Some(coords) = self.mouse_coords {
            if coords.x > window_position.x + WIDTH
                && window_position.x > -WIDTH {
                return true;
            }
        };

        false
    }

    pub fn slide_window(&mut self, main_window: &tauri::WebviewWindow) {
        let window_position = main_window
            .outer_position()
            .unwrap()
            .to_logical::<f64>(main_window.scale_factor().unwrap());
        let (x, y) = (window_position.x, window_position.y);
        if self.required_to_show_window(main_window) {
            main_window.set_position(
                tauri::Position::Logical(
                    LogicalPosition::new(x + VELOCITY, y)
                )
            ).unwrap();
        } else if self.required_to_hide_window(main_window) {
            main_window.set_position(
                tauri::Position::Logical(
                    LogicalPosition::new(x - VELOCITY, y)
                )
            ).unwrap();
        }
    }
}
