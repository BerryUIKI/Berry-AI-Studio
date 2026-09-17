mod commands;

use std::sync::Mutex;

use berry_storage::Database;
use tauri::Manager;

/// Application-wide state managed by Tauri.
pub struct AppState {
    /// The migrated SQLite database, opened in the app data directory.
    pub db: Mutex<Database>,
    /// Optional active WD14 ONNX Tagger instance.
    pub tagger: Mutex<Option<berry_tagger::Wd14Tagger>>,
    /// Optional active CLIP / SigLIP text & image embedding engine.
    pub clip: Mutex<Option<berry_clip::ClipEngine>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Open (and migrate) the SQLite database in the OS app data dir.
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let _ = std::fs::create_dir_all(data_dir.join("updates"));
            let _ = std::fs::create_dir_all(data_dir.join("thumbnails"));
            let _ = std::fs::create_dir_all(data_dir.join("models"));
            let db = Database::connect(&data_dir.join("berry.db"))?;
            app.manage(AppState {
                db: Mutex::new(db),
                tagger: Mutex::new(None),
                clip: Mutex::new(None),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_info,
            commands::add_folder,
            commands::list_folders,
            commands::remove_folder,
            commands::list_files,
            commands::query_files,
            commands::set_file_rating,
            commands::set_files_rating,
            commands::get_library_counts,
            commands::scan_folder,
            commands::rebuild_metadata,
            commands::search_files,
            commands::search_files_page,
            commands::search_files_by_query,
            commands::search_files_by_query_page,
            commands::list_distinct_models,
            commands::list_distinct_samplers,
            commands::create_album,
            commands::get_album,
            commands::list_albums,
            commands::rename_album,
            commands::delete_album,
            commands::add_file_to_album,
            commands::add_files_to_album,
            commands::remove_file_from_album,
            commands::remove_files_from_album,
            commands::count_album_files,
            commands::list_album_files,
            commands::create_tag,
            commands::list_tags,
            commands::delete_tag,
            commands::tag_file,
            commands::tag_files,
            commands::untag_file,
            commands::untag_files,
            commands::get_file_tags,
            commands::list_files_by_tag,
            commands::set_file_favorite,
            commands::set_files_favorite,
            commands::set_file_nsfw,
            commands::set_files_nsfw,
            commands::get_prompt_stats,
            commands::get_checkpoint_models,
            commands::import_model_cache_file,
            commands::resolve_model_hash,
            commands::list_model_cache,
            commands::move_files,
            commands::copy_files,
            commands::trash_files,
            commands::reveal_in_file_manager,
            commands::vacuum_database,
            commands::backup_database,
            commands::get_database_stats,
            commands::restore_database,
            commands::open_external_url,
            commands::get_or_create_thumbnail,
            commands::batch_generate_thumbnails,
            commands::get_thumbnail_cache_stats,
            commands::clear_thumbnail_cache,
            commands::upsert_file_embedding,
            commands::remove_file_embedding,
            commands::get_file_embedding,
            commands::get_file_embedding_models,
            commands::search_similar_files,
            commands::find_similar_to_file,
            commands::list_tagger_models,
            commands::load_tagger_model,
            commands::get_loaded_tagger_model,
            commands::auto_tag_file,
            commands::batch_auto_tag_files,
            commands::list_clip_models,
            commands::load_clip_model,
            commands::get_loaded_clip_model,
            commands::get_clip_index_status,
            commands::index_clip_images_batch,
            commands::search_by_text_prompt,
            commands::list_loras,
            commands::get_lora,
            commands::save_lora,
            commands::delete_lora,
            commands::get_image_detected_loras,
            commands::import_lora_civitai_info,
            commands::scan_loras_directory,
            commands::get_app_config,
            commands::save_app_config,
            commands::get_storage_paths,
            commands::open_storage_dir,
            commands::download_update,
            commands::install_update,
            commands::add_folder_with_options,
            commands::autodetect_local_ai_paths,
            commands::harvest_pipeline_folder,
            commands::process_pipeline_cleanups,
            commands::get_pipeline_cleanup_queue,
            commands::stack_images,
            commands::merge_stacks,
            commands::unstack_images,
            commands::set_stack_hero,
            commands::get_stack_members,
            commands::list_stacks,
            commands::cull_stack_drafts,
            commands::auto_stack_images,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
