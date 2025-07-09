use device_query::{DeviceQuery, DeviceState};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub enum MouseEvent {
    ButtonPressed(()),
    ButtonReleased(()),
    Move(()),
    TextSelected(String),
    ClickOutsideIcon(i32, i32), // x, y coordinates of the click
}

pub struct MouseService {
    running: Arc<AtomicBool>,
    is_mouse_down: Arc<AtomicBool>,
    start_pos: Arc<std::sync::Mutex<(i32, i32)>>,
    last_click: Arc<std::sync::Mutex<Option<Instant>>>,
    last_text_selection: Arc<std::sync::Mutex<Option<Instant>>>,
    // Track selection icon position and visibility
    selection_icon_bounds: Arc<std::sync::Mutex<Option<(i32, i32, i32, i32)>>>, // x, y, width, height
    selection_icon_visible: Arc<AtomicBool>,
}

impl MouseService {
    pub fn new() -> Self {
        MouseService {
            running: Arc::new(AtomicBool::new(false)),
            is_mouse_down: Arc::new(AtomicBool::new(false)),
            start_pos: Arc::new(std::sync::Mutex::new((0, 0))),
            last_click: Arc::new(std::sync::Mutex::new(None)),
            last_text_selection: Arc::new(std::sync::Mutex::new(None)),
            selection_icon_bounds: Arc::new(std::sync::Mutex::new(None)),
            selection_icon_visible: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn set_selection_icon_bounds(&self, x: i32, y: i32, width: i32, height: i32) {
        *self.selection_icon_bounds.lock().unwrap() = Some((x, y, width, height));
    }

    pub fn set_selection_icon_visible(&self, visible: bool) {
        self.selection_icon_visible.store(visible, Ordering::SeqCst);
    }

    fn is_click_inside_icon(&self, click_x: i32, click_y: i32) -> bool {
        if let Some((icon_x, icon_y, width, height)) = *self.selection_icon_bounds.lock().unwrap() {
            click_x >= icon_x && click_x <= icon_x + width &&
            click_y >= icon_y && click_y <= icon_y + height
        } else {
            false
        }
    }
    
    #[allow(unused_unsafe)]
    async fn check_for_selected_text(app_handle: &tauri::AppHandle, last_text: &mut String) -> Option<String> {
        if let Ok(selected_text) = unsafe { crate::selected_text::get_selected_text(app_handle).await } {
            if !selected_text.is_empty() && selected_text != *last_text {
                *last_text = selected_text.clone();
                return Some(selected_text);
            }
        }
        None
    }

    async fn is_double_click_enabled(app_handle: &tauri::AppHandle) -> bool {
        match crate::commands::get_double_click_enabled(app_handle.clone()).await {
            Ok(enabled) => enabled,
            Err(_) => false,
        }
    }

    pub fn start(&self, app_handle: tauri::AppHandle, callback: impl Fn(MouseEvent) + Send + 'static) {
        let running = self.running.clone();
        let is_mouse_down = self.is_mouse_down.clone();
        let start_pos = self.start_pos.clone();
        let last_click = self.last_click.clone();
        let last_text_selection = self.last_text_selection.clone();
        let selection_icon_bounds = self.selection_icon_bounds.clone();
        let selection_icon_visible = self.selection_icon_visible.clone();
        running.store(true, Ordering::SeqCst);

        thread::spawn(move || {
            let device_state = DeviceState::new();
            let mut last_state = device_state.get_mouse();
            let mut last_text = String::new();
            const DOUBLE_CLICK_DURATION: Duration = Duration::from_millis(500);
            const TEXT_SELECTION_DEBOUNCE: Duration = Duration::from_millis(300);
            
            // Helper function to check if we should emit text selection (debouncing)
            let should_emit_text_selection = |last_selection: &Arc<std::sync::Mutex<Option<Instant>>>| -> bool {
                let mut last_selection_guard = last_selection.lock().unwrap();
                if let Some(last_time) = *last_selection_guard {
                    if last_time.elapsed() < TEXT_SELECTION_DEBOUNCE {
                        return false; // Too soon, skip this selection
                    }
                }
                *last_selection_guard = Some(Instant::now());
                true
            };

            while running.load(Ordering::SeqCst) {
                let current_state = device_state.get_mouse();

                // Check for button presses
                for (index, &is_pressed) in current_state.button_pressed.iter().enumerate() {
                    if is_pressed && !last_state.button_pressed[index] {
                        if index == 1 { // Left button
                            is_mouse_down.store(true, Ordering::SeqCst);
                            *start_pos.lock().unwrap() = current_state.coords;
                            
                            // Check for double-click
                            let mut last_click_guard = last_click.lock().unwrap();
                            if let Some(last_time) = *last_click_guard {
                                if last_time.elapsed() < DOUBLE_CLICK_DURATION {
                                    // Check if double-click is enabled before processing
                                    let double_click_enabled = tauri::async_runtime::block_on(async {
                                        Self::is_double_click_enabled(&app_handle).await
                                    });
                                    
                                    if double_click_enabled {
                                        // Double click detected and enabled, check for text selection with debouncing
                                        if should_emit_text_selection(&last_text_selection) {
                                            if let Some(selected_text) = tauri::async_runtime::block_on(async {
                                                Self::check_for_selected_text(&app_handle, &mut last_text).await
                                            }) {
                                                callback(MouseEvent::TextSelected(selected_text));
                                            }
                                        }
                                    }
                                    *last_click_guard = None;
                                } else {
                                    *last_click_guard = Some(Instant::now());
                                }
                            } else {
                                *last_click_guard = Some(Instant::now());
                            }

                            // Check if this is a click outside the selection icon
                            if selection_icon_visible.load(Ordering::SeqCst) {
                                let (click_x, click_y) = current_state.coords;
                                let is_inside = if let Some((icon_x, icon_y, width, height)) = *selection_icon_bounds.lock().unwrap() {
                                    click_x >= icon_x && click_x <= icon_x + width &&
                                    click_y >= icon_y && click_y <= icon_y + height
                                } else {
                                    false
                                };

                                if !is_inside {
                                    callback(MouseEvent::ClickOutsideIcon(click_x, click_y));
                                }
                            }
                        }
                        callback(MouseEvent::ButtonPressed(()));
                    }
                }

                // Check for button releases
                for (index, &is_pressed) in current_state.button_pressed.iter().enumerate() {
                    if !is_pressed && last_state.button_pressed[index] {
                        if index == 1 { // Left button
                            is_mouse_down.store(false, Ordering::SeqCst);
                            let end_pos = current_state.coords;
                            let start = *start_pos.lock().unwrap();
                            
                            // Check for drag selection
                            if start != end_pos {
                                // Check if double-click is enabled before processing drag text selection
                                let double_click_enabled = tauri::async_runtime::block_on(async {
                                    Self::is_double_click_enabled(&app_handle).await
                                });
                                
                                if double_click_enabled {
                                    // Small delay to allow OS to update selection
                                    thread::sleep(Duration::from_millis(50));
                                    #[cfg(target_os = "windows")]
                                    let selected_text = tauri::async_runtime::block_on(async {
                                        unsafe { crate::selected_text::get_selected_text(&app_handle).await }
                                    });

                                    #[cfg(target_os = "macos")]
                                    let selected_text = tauri::async_runtime::block_on(async {
                                        crate::selected_text::get_selected_text(&app_handle).await
                                    });

                                    if let Ok(selected_text) = selected_text {
                                        if !selected_text.is_empty() && selected_text != last_text {
                                            if should_emit_text_selection(&last_text_selection) {
                                                last_text = selected_text.clone();
                                                callback(MouseEvent::TextSelected(selected_text));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        callback(MouseEvent::ButtonReleased(()));
                    }
                }

                // Detect movement while mouse is down
                if is_mouse_down.load(Ordering::SeqCst) && current_state.coords != last_state.coords {
                    callback(MouseEvent::Move(()));
                }

                last_state = current_state;
                thread::sleep(Duration::from_millis(10));
            }
        });
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
} 