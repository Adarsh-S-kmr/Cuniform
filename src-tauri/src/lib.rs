use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, SetWindowPos, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, GetWindowThreadProcessId,
    IsIconic, ShowWindow, SW_MINIMIZE, SW_RESTORE, IsWindow
};

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::State;
use uuid::Uuid;

#[derive(Clone, serde::Serialize)]
struct Tile {
    id: String,
    name: String,
    hwnds: Vec<isize>,
    #[serde(skip)]
    is_minimized: bool,
}

struct AppState {
    tiles: Arc<Mutex<Vec<Tile>>>,
    last_external_hwnd: Arc<Mutex<isize>>,
}

#[tauri::command]
fn exit_app(app_handle: tauri::AppHandle) {
    app_handle.exit(0);
}

#[tauri::command]
fn create_tile(state: State<'_, AppState>) -> Result<String, String> {
    let mut tiles = state.tiles.lock().unwrap();
    let new_id = Uuid::new_v4().to_string();
    let name = format!("Tile {}", &new_id[..6]);
    tiles.push(Tile {
        id: new_id.clone(),
        name,
        hwnds: Vec::new(),
        is_minimized: false,
    });
    Ok(new_id)
}

#[tauri::command]
fn capture_active_window_to_tile(tile_id: String, state: State<'_, AppState>) -> Result<String, String> {
    let hwnd_val = *state.last_external_hwnd.lock().unwrap();
    if hwnd_val == 0 {
        return Err("No external window found".into());
    }
    let hwnd = HWND(hwnd_val as *mut std::ffi::c_void);
    
    let mut tiles = state.tiles.lock().unwrap();
    if let Some(tile) = tiles.iter_mut().find(|t| t.id == tile_id) {
        if !tile.hwnds.contains(&(hwnd.0 as isize)) {
            tile.hwnds.push(hwnd.0 as isize);
            
            // Phase 4 preview: execute layout!
            apply_tile_layout(&tile.hwnds);
        }
        Ok(format!("Added window to tile {}", tile_id))
    } else {
        Err("Tile not found".into())
    }
}

#[tauri::command]
fn dissolve_tile(tile_id: String, state: State<'_, AppState>) -> Result<String, String> {
    let mut tiles = state.tiles.lock().unwrap();
    let initial_len = tiles.len();
    tiles.retain(|t| t.id != tile_id);
    if tiles.len() < initial_len {
        Ok(format!("Dissolved tile {}", tile_id))
    } else {
        Err("Tile not found".into())
    }
}

#[tauri::command]
fn get_tiles(state: State<'_, AppState>) -> Result<Vec<Tile>, String> {
    let tiles = state.tiles.lock().unwrap();
    Ok(tiles.clone())
}

#[tauri::command]
fn toggle_tile_visibility(tile_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut tiles = state.tiles.lock().unwrap();
    if let Some(tile) = tiles.iter_mut().find(|t| t.id == tile_id) {
        let target_minimized = !tile.is_minimized;
        tile.is_minimized = target_minimized;
        
        for &hwnd_val in &tile.hwnds {
            let hwnd = HWND(hwnd_val as *mut std::ffi::c_void);
            unsafe {
                if target_minimized {
                    let _ = ShowWindow(hwnd, SW_MINIMIZE);
                } else {
                    let _ = ShowWindow(hwnd, SW_RESTORE);
                }
            }
        }
        Ok(())
    } else {
        Err("Tile not found".into())
    }
}

#[tauri::command]
fn update_tile_name(tile_id: String, name: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut tiles = state.tiles.lock().unwrap();
    if let Some(tile) = tiles.iter_mut().find(|t| t.id == tile_id) {
        tile.name = name;
        Ok(())
    } else {
        Err("Tile not found".into())
    }
}

use windows::Win32::UI::WindowsAndMessaging::{SM_CXSCREEN, SM_CYSCREEN, GetSystemMetrics};

fn apply_tile_layout(hwnds: &[isize]) {
    if hwnds.is_empty() { return; }
    
    let screen_w = unsafe { GetSystemMetrics(SM_CXSCREEN) };
    let screen_h = unsafe { GetSystemMetrics(SM_CYSCREEN) };
    let count = hwnds.len() as i32;
    
    if count == 4 {
        let width = screen_w / 2;
        let height = screen_h / 2;
        for (i, &hwnd_val) in hwnds.iter().enumerate() {
            let hwnd = HWND(hwnd_val as *mut std::ffi::c_void);
            let row = (i as i32) / 2;
            let col = (i as i32) % 2;
            unsafe {
                let _ = SetWindowPos(
                    hwnd, 
                    Some(HWND(std::ptr::null_mut())), 
                    col * width, 
                    row * height, 
                    width, 
                    height, 
                    SWP_NOZORDER
                );
            }
        }
    } else {
        let width = screen_w / count;
        
        for (i, &hwnd_val) in hwnds.iter().enumerate() {
            let hwnd = HWND(hwnd_val as *mut std::ffi::c_void);
            unsafe {
                let _ = SetWindowPos(
                    hwnd, 
                    Some(HWND(std::ptr::null_mut())), 
                    i as i32 * width, 
                    0, 
                    width, 
                    screen_h, 
                    SWP_NOZORDER
                );
            }
        }
    }
}

use tauri::Manager;
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let shortcut = Shortcut::new(Some(Modifiers::ALT | Modifiers::SHIFT), Code::KeyC);

    let tiles = Arc::new(Mutex::new(Vec::<Tile>::new()));
    let tiles_clone = tiles.clone();
    
    let last_external_hwnd = Arc::new(Mutex::new(0isize));
    let last_external_clone = last_external_hwnd.clone();

    thread::spawn(move || {
        let mut last_foreground: isize = 0;
        
        loop {
            let hwnd = unsafe { GetForegroundWindow() };
            let current = hwnd.0 as isize;
            
            if current != 0 {
                let mut pid: u32 = 0;
                unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)); }
                if pid != std::process::id() {
                    if let Ok(mut last_ext) = last_external_clone.lock() {
                        *last_ext = current;
                    }
                }
                
                let tiles_guard = tiles_clone.lock().unwrap();
                if let Some(tile) = tiles_guard.iter().find(|t| t.hwnds.contains(&current)) {
                    if current != last_foreground {
                        last_foreground = current;
                        for &other_val in &tile.hwnds {
                            if other_val != current {
                                let other_hwnd = HWND(other_val as *mut std::ffi::c_void);
                                unsafe {
                                    let _ = SetWindowPos(
                                        other_hwnd,
                                        Some(HWND(std::ptr::null_mut())),
                                        0, 0, 0, 0,
                                        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE
                                    );
                                }
                            }
                        }
                    }
                } else {
                    last_foreground = current;
                }
            }
            
            // Prune dead windows
            {
                let mut tiles_guard = tiles_clone.lock().unwrap();
                for tile in tiles_guard.iter_mut() {
                    tile.hwnds.retain(|&hwnd_val| {
                        let hwnd = HWND(hwnd_val as *mut std::ffi::c_void);
                        unsafe { IsWindow(Some(hwnd)).as_bool() }
                    });
                }
            }
            
            thread::sleep(Duration::from_millis(10));
        }
    });

    tauri::Builder::default()
        .manage(AppState {
            tiles,
            last_external_hwnd,
        })
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_shortcuts([shortcut])
                .expect("failed to register shortcut")
                .with_handler(move |app, active_shortcut, event| {
                    if active_shortcut == &shortcut {
                        if event.state == ShortcutState::Pressed {
                            if let Some(window) = app.get_webview_window("main") {
                                if window.is_visible().unwrap_or(false) {
                                    let _ = window.hide();
                                } else {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            }
                        }
                    }
                })
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            create_tile, 
            capture_active_window_to_tile, 
            dissolve_tile, 
            get_tiles,
            toggle_tile_visibility,
            update_tile_name,
            exit_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
