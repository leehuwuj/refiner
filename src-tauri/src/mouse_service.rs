use device_query::{DeviceQuery, DeviceState, MouseState};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tauri::Manager;

#[derive(Debug)]
pub enum MouseEvent {
    ButtonPressed(usize),
    ButtonReleased(usize),
    Move((i32, i32)),
    TextSelected(String),
}

pub struct MouseService {
    running: Arc<AtomicBool>,
    is_mouse_down: Arc<AtomicBool>,
    start_pos: Arc<std::sync::Mutex<(i32, i32)>>,
    last_click: Arc<std::sync::Mutex<Option<Instant>>>,
}

impl MouseService {
    pub fn new() -> Self {
        MouseService {
            running: Arc::new(AtomicBool::new(false)),
            is_mouse_down: Arc::new(AtomicBool::new(false)),
            start_pos: Arc::new(std::sync::Mutex::new((0, 0))),
            last_click: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    async fn check_for_selected_text(app_handle: &tauri::AppHandle, last_text: &mut String) -> Option<String> {
        if let Ok(selected_text) = crate::selected_text::get_selected_text(app_handle).await {
            if !selected_text.is_empty() && selected_text != *last_text {
                *last_text = selected_text.clone();
                return Some(selected_text);
            }
        }
        None
    }

    pub fn start(&self, app_handle: tauri::AppHandle, callback: impl Fn(MouseEvent) + Send + 'static) {
        let running = self.running.clone();
        let is_mouse_down = self.is_mouse_down.clone();
        let start_pos = self.start_pos.clone();
        let last_click = self.last_click.clone();
        running.store(true, Ordering::SeqCst);

        thread::spawn(move || {
            let device_state = DeviceState::new();
            let mut last_state = device_state.get_mouse();
            let mut last_text = String::new();
            const DOUBLE_CLICK_DURATION: Duration = Duration::from_millis(500);

            while running.load(Ordering::SeqCst) {
                let current_state = device_state.get_mouse();
                
                for (index, (&last_pressed, &current_pressed)) in last_state.button_pressed.iter().zip(current_state.button_pressed.iter()).enumerate() {
                    if !last_pressed && current_pressed {
                        if index == 1 { // Left button
                            is_mouse_down.store(true, Ordering::SeqCst);
                            *start_pos.lock().unwrap() = current_state.coords;

                            // Check for double click
                            let mut last_click_guard = last_click.lock().unwrap();
                            if let Some(last_click_time) = *last_click_guard {
                                if last_click_time.elapsed() < DOUBLE_CLICK_DURATION {
                                    // Double click detected, check for text selection
                                    if let Some(selected_text) = tauri::async_runtime::block_on(async {
                                        Self::check_for_selected_text(&app_handle, &mut last_text).await
                                    }) {
                                        callback(MouseEvent::TextSelected(selected_text));
                                    }
                                    *last_click_guard = None;
                                } else {
                                    *last_click_guard = Some(Instant::now());
                                }
                            } else {
                                *last_click_guard = Some(Instant::now());
                            }
                        }
                        callback(MouseEvent::ButtonPressed(index));
                    } else if last_pressed && !current_pressed {
                        if index == 1 { // Left button
                            is_mouse_down.store(false, Ordering::SeqCst);
                            let end_pos = current_state.coords;
                            let start = *start_pos.lock().unwrap();
                            
                            // Check for drag selection
                            if start != end_pos {
                                // Small delay to allow OS to update selection
                                thread::sleep(Duration::from_millis(50));
                                if let Some(selected_text) = tauri::async_runtime::block_on(async {
                                    Self::check_for_selected_text(&app_handle, &mut last_text).await
                                }) {
                                    callback(MouseEvent::TextSelected(selected_text));
                                }
                            }
                        }
                        callback(MouseEvent::ButtonReleased(index));
                    }
                }

                // Detect movement while mouse is down
                if is_mouse_down.load(Ordering::SeqCst) && current_state.coords != last_state.coords {
                    callback(MouseEvent::Move(current_state.coords));
                }

                last_state = current_state;
                thread::sleep(Duration::from_millis(10));
            }
        });
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    pub fn is_selecting(&self) -> bool {
        self.is_mouse_down.load(Ordering::SeqCst)
    }
} 