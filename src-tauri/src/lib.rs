mod scanner;
mod api_client;
mod settings;

use scanner::{AudioFile, FileScanner, Metadata};
use api_client::{SpotifyClient, MusicBrainzClient, BeatportClient};
use settings::{save_settings, load_settings};
use std::path::PathBuf;
use id3::TagLike;

#[tauri::command]
fn scan_folder(path: String) -> Result<Vec<AudioFile>, String> {
    let scanner = FileScanner::new();
    let folder_path = PathBuf::from(path);
    scanner.scan_directory(&folder_path)
}

#[tauri::command]
async fn fetch_metadata(app: tauri::AppHandle, artist: String, title: String) -> Result<Vec<api_client::MetadataResult>, String> {
    let mut results = Vec::new();
    
    let settings = load_settings(app.clone()).ok();
    
    let client_id = std::env::var("SPOTIFY_CLIENT_ID")
        .ok()
        .or_else(|| {
            settings.as_ref()
                .and_then(|s| if s.spotify_client_id.is_empty() { None } else { Some(s.spotify_client_id.clone()) })
        });
    
    let client_secret = std::env::var("SPOTIFY_CLIENT_SECRET")
        .ok()
        .or_else(|| {
            settings.as_ref()
                .and_then(|s| if s.spotify_client_secret.is_empty() { None } else { Some(s.spotify_client_secret.clone()) })
        });
    
    let beatport_username = std::env::var("BEATPORT_USERNAME").ok();
    let beatport_password = std::env::var("BEATPORT_PASSWORD").ok();
    
    let spotify_client = SpotifyClient::new(client_id, client_secret);
    if let Ok(result) = spotify_client.search_track(&artist, &title).await {
        results.push(result);
    }
    
    let beatport_client = BeatportClient::new(beatport_username, beatport_password);
    if let Ok(result) = beatport_client.search_track(&artist, &title).await {
        results.push(result);
    }
    
    let mb_client = MusicBrainzClient::new();
    if let Ok(result) = mb_client.search_track(&artist, &title).await {
        results.push(result);
    }
    
    Ok(results)
}

#[tauri::command]
fn update_metadata(file_path: String, metadata: Metadata, backup: bool) -> Result<(), String> {
    let scanner = FileScanner::new();
    let path = PathBuf::from(&file_path);
    
    if backup {
        let ext = path.extension().and_then(|s| s.to_str());
        let current_metadata = match ext {
            Some("mp3") => {
                id3::Tag::read_from_path(&path).ok().map(|tag| Metadata {
                    title: tag.title().map(|s| s.to_string()),
                    artist: tag.artist().map(|s| s.to_string()),
                    album: tag.album().map(|s| s.to_string()),
                    genre: tag.genre().map(|s| s.to_string()),
                    year: tag.year(),
                    bpm: None,
                })
            },
            Some("flac") | Some("wav") | Some("ogg") | Some("m4a") => {
                use lofty::prelude::*;
                use lofty::config::ParseOptions;
                use lofty::probe::Probe;
                Probe::open(&path)
                    .ok()
                    .and_then(|probe| probe.options(ParseOptions::new()).read().ok())
                    .and_then(|file| file.primary_tag().or_else(|| file.first_tag()).map(|tag| Metadata {
                        title: tag.title().map(|s| s.to_string()),
                        artist: tag.artist().map(|s| s.to_string()),
                        album: tag.album().map(|s| s.to_string()),
                        genre: tag.genre().map(|s| s.to_string()),
                        year: tag.year().map(|y| y as i32),
                        bpm: None,
                    }))
            },
            _ => None
        };
        
        if let Some(current) = current_metadata {
            scanner.backup_metadata(&path, &current)?;
        } else {
            return Err("Cannot read current metadata for backup".to_string());
        }
    }
    
    scanner.write_metadata(&path, &metadata)?;
    Ok(())
}

#[tauri::command]
fn organize_files(app: tauri::AppHandle, file_path: String, metadata: Metadata, base_folder: String) -> Result<String, String> {
    let scanner = FileScanner::new();
    let path = PathBuf::from(file_path);
    let base = PathBuf::from(base_folder);
    
    let settings = load_settings(app)?;
    let new_path = scanner.organize_file(&path, &metadata, &base, &settings.folder_pattern)?;
    Ok(new_path.to_string_lossy().to_string())
}

#[tauri::command]
fn rename_file(file_path: String, metadata: Metadata) -> Result<String, String> {
    let scanner = FileScanner::new();
    let path = PathBuf::from(file_path);
    
    let new_path = scanner.rename_file(&path, &metadata)?;
    Ok(new_path.to_string_lossy().to_string())
}

#[tauri::command]
fn restore_from_backup(backup_path: String, original_path: String) -> Result<(), String> {
    let scanner = FileScanner::new();
    let backup = PathBuf::from(backup_path);
    let original = PathBuf::from(original_path);
    
    scanner.restore_from_backup(&backup, &original)?;
    Ok(())
}

#[tauri::command]
fn find_duplicates(files: Vec<AudioFile>) -> Vec<Vec<usize>> {
    let scanner = FileScanner::new();
    scanner.find_duplicates(&files)
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            scan_folder, 
            fetch_metadata,
            update_metadata,
            organize_files,
            rename_file,
            restore_from_backup,
            find_duplicates,
            save_settings,
            load_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
