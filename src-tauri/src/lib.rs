mod clipboard;
use clipboard::test;


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    test();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_prevent_default::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
