pub mod document;
pub mod file_ops;
pub mod workspace;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            file_ops::create_file,
            file_ops::create_folder,
            file_ops::delete_file,
            file_ops::read_file_content,
            file_ops::write_file_content,
            file_ops::is_file_binary,
            file_ops::delete_folder,
            file_ops::rename_entry,
            file_ops::copy_entry,
            file_ops::move_entry,
            workspace::get_recent_workspaces,
            workspace::create_workspace,
            workspace::add_file_to_workspace,
            workspace::get_workspace_files,
            workspace::save_annotations,
            workspace::load_annotations,
            workspace::read_workspace_file
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            app.handle().plugin(tauri_plugin_dialog::init())?;
            app.handle().plugin(tauri_plugin_fs::init())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
