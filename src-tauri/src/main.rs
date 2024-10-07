// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{LogicalPosition, LogicalSize, Manager, Size, ActivationPolicy};
use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial, NSVisualEffectState};
use std::time::{Duration, Instant};
use tauri_plugin_positioner::WindowExt;

const HEIGHT: f64 = 800.0;
const WIDTH: f64 = 180.0;
const MAX_TOUCH_COUNT: u32 = 4;
const UPDATE_INTERVAL: Duration = Duration::from_millis(1000 / 15);


#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() {

    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .setup(|app| {

            app.set_activation_policy(ActivationPolicy::Accessory);
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
             

                // observe mouse
                loop {
                    state.update(&main_window);
                    state.adjust_window(&main_window);
                    state.wait_next_update();
                }

            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

}

pub struct AppState {
    mouse_coords: Option<LogicalPosition<f64>>,
    time_instant: Instant,
    touch_edge_count: u32,
    touch_outside_count: u32,
    is_window_shown: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            mouse_coords: None,
            time_instant: Instant::now(),
            touch_edge_count: 0,
            touch_outside_count: 0,
            is_window_shown: false,
        }
    }

    pub fn update(&mut self, main_window: &tauri::WebviewWindow) {
        self.time_instant = Instant::now();
        self.mouse_coords = if let Ok(physical_position) = main_window.cursor_position() {
            Some(physical_position.to_logical::<f64>(main_window.scale_factor().unwrap()))
        } else {
            None
        };

        if let Some(coords) = self.mouse_coords {
            if coords.x == 0.0 {
                self.touch_edge_count += 1;
                self.touch_outside_count = 0;
            } else {
                self.touch_edge_count = 0;
                self.touch_outside_count += 1;
            }
        };
    }

    pub fn wait_next_update(&mut self) {
        let elapsed = self.time_instant.elapsed();
        if elapsed < UPDATE_INTERVAL {
            std::thread::sleep(UPDATE_INTERVAL - elapsed);
        }
    }

    pub fn required_to_show_window(&mut self) -> bool {
        if let Some(coords) = self.mouse_coords {
            if coords.x == 0.0 && self.touch_edge_count >= MAX_TOUCH_COUNT && !self.is_window_shown {
                return true;
            }
        };

        false
    }

    pub fn required_to_hide_window(&mut self) -> bool {
        if let Some(coords) = self.mouse_coords {
            if coords.x > WIDTH && self.touch_outside_count >= MAX_TOUCH_COUNT && self.is_window_shown {
                return true;
            }
        };

        false
    }

    pub fn adjust_window(&mut self, main_window: &tauri::WebviewWindow) {
        if self.required_to_show_window() {
            let _ = main_window.show();
            self.is_window_shown = true;
            println!("Show");
        } else if self.required_to_hide_window() {
            let _ = main_window.hide();
            self.is_window_shown = false;
            println!("Hide");
        }
    }
}
