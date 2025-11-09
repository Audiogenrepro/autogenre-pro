use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use walkdir::WalkDir;
use id3::TagLike;
use lofty::prelude::*;
use lofty::config::{ParseOptions, WriteOptions};
use lofty::probe::Probe;
use lofty::tag::{Tag, TagType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioFile {
    pub path: PathBuf,
    pub filename: String,
    pub extension: String,
    pub current_metadata: Option<Metadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub genre: Option<String>,
    pub year: Option<i32>,
    pub bpm: Option<f32>,
}

pub struct FileScanner {
    supported_extensions: Vec<String>,
}

impl FileScanner {
    pub fn new() -> Self {
        FileScanner {
            supported_extensions: vec![
                "mp3".to_string(),
                "flac".to_string(),
                "wav".to_string(),
                "m4a".to_string(),
                "aiff".to_string(),
                "ogg".to_string(),
            ],
        }
    }

    pub fn scan_directory(&self, path: &Path) -> Result<Vec<AudioFile>, String> {
        let mut audio_files = Vec::new();

        for entry in WalkDir::new(path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let file_path = entry.path();
            
            if file_path.is_file() {
                if let Some(extension) = file_path.extension() {
                    let ext = extension.to_string_lossy().to_lowercase();
                    
                    if self.supported_extensions.contains(&ext) {
                        let filename = file_path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();

                        let audio_file = AudioFile {
                            path: file_path.to_path_buf(),
                            filename,
                            extension: ext,
                            current_metadata: self.read_metadata(file_path).ok(),
                        };

                        audio_files.push(audio_file);
                    }
                }
            }
        }

        Ok(audio_files)
    }

    fn read_metadata(&self, path: &Path) -> Result<Metadata, String> {
        let ext = path.extension().and_then(|s| s.to_str());
        match ext {
            Some("mp3") => self.read_mp3_metadata(path),
            Some("flac") => self.read_flac_metadata(path),
            Some("wav") => self.read_wav_metadata(path),
            Some("ogg") => self.read_ogg_metadata(path),
            Some("m4a") => self.read_m4a_metadata(path),
            _ => Ok(Metadata {
                title: None,
                artist: None,
                album: None,
                genre: None,
                year: None,
                bpm: None,
            })
        }
    }

    fn read_mp3_metadata(&self, path: &Path) -> Result<Metadata, String> {
        let tag = id3::Tag::read_from_path(path)
            .map_err(|e| format!("Failed to read ID3 tags: {}", e))?;

        Ok(Metadata {
            title: tag.title().map(|s| s.to_string()),
            artist: tag.artist().map(|s| s.to_string()),
            album: tag.album().map(|s| s.to_string()),
            genre: tag.genre().map(|s| s.to_string()),
            year: tag.year(),
            bpm: None,
        })
    }

    fn read_flac_metadata(&self, path: &Path) -> Result<Metadata, String> {
        let tagged_file = Probe::open(path)
            .map_err(|e| format!("Failed to open FLAC file: {}", e))?
            .options(ParseOptions::new())
            .read()
            .map_err(|e| format!("Failed to read FLAC file: {}", e))?;

        let tag = tagged_file.primary_tag()
            .or_else(|| tagged_file.first_tag())
            .ok_or("No tags found in FLAC file")?;

        Ok(Metadata {
            title: tag.title().map(|s| s.to_string()),
            artist: tag.artist().map(|s| s.to_string()),
            album: tag.album().map(|s| s.to_string()),
            genre: tag.genre().map(|s| s.to_string()),
            year: tag.year().map(|y| y as i32),
            bpm: None,
        })
    }

    fn read_wav_metadata(&self, path: &Path) -> Result<Metadata, String> {
        let tagged_file = Probe::open(path)
            .map_err(|e| format!("Failed to open WAV file: {}", e))?
            .options(ParseOptions::new())
            .read()
            .map_err(|e| format!("Failed to read WAV file: {}", e))?;

        let tag = tagged_file.primary_tag()
            .or_else(|| tagged_file.first_tag())
            .ok_or("No tags found in WAV file")?;

        Ok(Metadata {
            title: tag.title().map(|s| s.to_string()),
            artist: tag.artist().map(|s| s.to_string()),
            album: tag.album().map(|s| s.to_string()),
            genre: tag.genre().map(|s| s.to_string()),
            year: tag.year().map(|y| y as i32),
            bpm: None,
        })
    }

    fn read_ogg_metadata(&self, path: &Path) -> Result<Metadata, String> {
        let tagged_file = Probe::open(path)
            .map_err(|e| format!("Failed to open OGG file: {}", e))?
            .options(ParseOptions::new())
            .read()
            .map_err(|e| format!("Failed to read OGG file: {}", e))?;

        let tag = tagged_file.primary_tag()
            .or_else(|| tagged_file.first_tag())
            .ok_or("No tags found in OGG file")?;

        Ok(Metadata {
            title: tag.title().map(|s| s.to_string()),
            artist: tag.artist().map(|s| s.to_string()),
            album: tag.album().map(|s| s.to_string()),
            genre: tag.genre().map(|s| s.to_string()),
            year: tag.year().map(|y| y as i32),
            bpm: None,
        })
    }

    fn read_m4a_metadata(&self, path: &Path) -> Result<Metadata, String> {
        let tagged_file = Probe::open(path)
            .map_err(|e| format!("Failed to open M4A file: {}", e))?
            .options(ParseOptions::new())
            .read()
            .map_err(|e| format!("Failed to read M4A file: {}", e))?;

        let tag = tagged_file.primary_tag()
            .or_else(|| tagged_file.first_tag())
            .ok_or("No tags found in M4A file")?;

        Ok(Metadata {
            title: tag.title().map(|s| s.to_string()),
            artist: tag.artist().map(|s| s.to_string()),
            album: tag.album().map(|s| s.to_string()),
            genre: tag.genre().map(|s| s.to_string()),
            year: tag.year().map(|y| y as i32),
            bpm: None,
        })
    }

    pub fn write_metadata(&self, path: &Path, metadata: &Metadata) -> Result<(), String> {
        let ext = path.extension().and_then(|s| s.to_str());
        match ext {
            Some("mp3") => self.write_mp3_metadata(path, metadata),
            Some("flac") => self.write_flac_metadata(path, metadata),
            Some("wav") => self.write_wav_metadata(path, metadata),
            Some("ogg") => self.write_ogg_metadata(path, metadata),
            Some("m4a") => self.write_m4a_metadata(path, metadata),
            _ => Err(format!("Unsupported file format for writing: {:?}", ext))
        }
    }

    fn write_mp3_metadata(&self, path: &Path, metadata: &Metadata) -> Result<(), String> {
        let mut tag = id3::Tag::read_from_path(path)
            .unwrap_or_else(|_| id3::Tag::new());

        if let Some(ref title) = metadata.title {
            tag.set_title(title);
        }

        if let Some(ref artist) = metadata.artist {
            tag.set_artist(artist);
        }

        if let Some(ref album) = metadata.album {
            tag.set_album(album);
        }

        if let Some(ref genre) = metadata.genre {
            tag.set_genre(genre);
        }

        if let Some(year) = metadata.year {
            tag.set_year(year);
        }

        tag.write_to_path(path, id3::Version::Id3v24)
            .map_err(|e| format!("Failed to write ID3 tags: {}", e))?;

        Ok(())
    }

    fn write_flac_metadata(&self, path: &Path, metadata: &Metadata) -> Result<(), String> {
        let mut tagged_file = Probe::open(path)
            .map_err(|e| format!("Failed to open FLAC file: {}", e))?
            .options(ParseOptions::new())
            .read()
            .map_err(|e| format!("Failed to read FLAC file: {}", e))?;

        let tag = match tagged_file.primary_tag_mut() {
            Some(t) => t,
            None => {
                let new_tag = Tag::new(TagType::VorbisComments);
                tagged_file.insert_tag(new_tag);
                tagged_file.primary_tag_mut()
                    .ok_or("Failed to create new tag")?
            }
        };

        if let Some(ref title) = metadata.title {
            tag.set_title(title.clone());
        }

        if let Some(ref artist) = metadata.artist {
            tag.set_artist(artist.clone());
        }

        if let Some(ref album) = metadata.album {
            tag.set_album(album.clone());
        }

        if let Some(ref genre) = metadata.genre {
            tag.set_genre(genre.clone());
        }

        if let Some(year) = metadata.year {
            tag.set_year(year as u32);
        }

        tag.save_to_path(path, WriteOptions::default())
            .map_err(|e| format!("Failed to write FLAC tags: {}", e))?;

        Ok(())
    }

    fn write_wav_metadata(&self, path: &Path, metadata: &Metadata) -> Result<(), String> {
        let mut tagged_file = Probe::open(path)
            .map_err(|e| format!("Failed to open WAV file: {}", e))?
            .options(ParseOptions::new())
            .read()
            .map_err(|e| format!("Failed to read WAV file: {}", e))?;

        let tag = match tagged_file.primary_tag_mut() {
            Some(t) => t,
            None => {
                let new_tag = Tag::new(TagType::Id3v2);
                tagged_file.insert_tag(new_tag);
                tagged_file.primary_tag_mut()
                    .ok_or("Failed to create new tag")?
            }
        };

        if let Some(ref title) = metadata.title {
            tag.set_title(title.clone());
        }

        if let Some(ref artist) = metadata.artist {
            tag.set_artist(artist.clone());
        }

        if let Some(ref album) = metadata.album {
            tag.set_album(album.clone());
        }

        if let Some(ref genre) = metadata.genre {
            tag.set_genre(genre.clone());
        }

        if let Some(year) = metadata.year {
            tag.set_year(year as u32);
        }

        tag.save_to_path(path, WriteOptions::default())
            .map_err(|e| format!("Failed to write WAV tags: {}", e))?;

        Ok(())
    }

    fn write_ogg_metadata(&self, path: &Path, metadata: &Metadata) -> Result<(), String> {
        let mut tagged_file = Probe::open(path)
            .map_err(|e| format!("Failed to open OGG file: {}", e))?
            .options(ParseOptions::new())
            .read()
            .map_err(|e| format!("Failed to read OGG file: {}", e))?;

        let tag = match tagged_file.primary_tag_mut() {
            Some(t) => t,
            None => {
                let new_tag = Tag::new(TagType::VorbisComments);
                tagged_file.insert_tag(new_tag);
                tagged_file.primary_tag_mut()
                    .ok_or("Failed to create new tag")?
            }
        };

        if let Some(ref title) = metadata.title {
            tag.set_title(title.clone());
        }

        if let Some(ref artist) = metadata.artist {
            tag.set_artist(artist.clone());
        }

        if let Some(ref album) = metadata.album {
            tag.set_album(album.clone());
        }

        if let Some(ref genre) = metadata.genre {
            tag.set_genre(genre.clone());
        }

        if let Some(year) = metadata.year {
            tag.set_year(year as u32);
        }

        tag.save_to_path(path, WriteOptions::default())
            .map_err(|e| format!("Failed to write OGG tags: {}", e))?;

        Ok(())
    }

    fn write_m4a_metadata(&self, path: &Path, metadata: &Metadata) -> Result<(), String> {
        let mut tagged_file = Probe::open(path)
            .map_err(|e| format!("Failed to open M4A file: {}", e))?
            .options(ParseOptions::new())
            .read()
            .map_err(|e| format!("Failed to read M4A file: {}", e))?;

        let tag = match tagged_file.primary_tag_mut() {
            Some(t) => t,
            None => {
                let new_tag = Tag::new(TagType::Mp4Ilst);
                tagged_file.insert_tag(new_tag);
                tagged_file.primary_tag_mut()
                    .ok_or("Failed to create new tag")?
            }
        };

        if let Some(ref title) = metadata.title {
            tag.set_title(title.clone());
        }

        if let Some(ref artist) = metadata.artist {
            tag.set_artist(artist.clone());
        }

        if let Some(ref album) = metadata.album {
            tag.set_album(album.clone());
        }

        if let Some(ref genre) = metadata.genre {
            tag.set_genre(genre.clone());
        }

        if let Some(year) = metadata.year {
            tag.set_year(year as u32);
        }

        tag.save_to_path(path, WriteOptions::default())
            .map_err(|e| format!("Failed to write M4A tags: {}", e))?;

        Ok(())
    }

    pub fn backup_metadata(&self, path: &Path, metadata: &Metadata) -> Result<PathBuf, String> {
        let backup_dir = path.parent()
            .ok_or("Cannot determine parent directory")?
            .join(".autogenre_backups");
        
        fs::create_dir_all(&backup_dir)
            .map_err(|e| format!("Failed to create backup directory: {}", e))?;

        let filename = path.file_name()
            .ok_or("Cannot determine filename")?
            .to_string_lossy();
        
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let backup_filename = format!("{}.{}.json", filename, timestamp);
        let backup_path = backup_dir.join(backup_filename);

        let json_data = serde_json::to_string_pretty(metadata)
            .map_err(|e| format!("Failed to serialize metadata: {}", e))?;

        fs::write(&backup_path, json_data)
            .map_err(|e| format!("Failed to write backup file: {}", e))?;

        Ok(backup_path)
    }

    pub fn organize_file(&self, path: &Path, metadata: &Metadata, base_folder: &Path, pattern: &str) -> Result<PathBuf, String> {
        let sanitize = |s: &str| -> String {
            s.chars()
                .map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' { c } else { '_' })
                .collect()
        };

        let expanded_pattern = pattern
            .replace("{genre}", &metadata.genre.as_ref().map(|g| sanitize(g)).unwrap_or_else(|| "Unknown".to_string()))
            .replace("{artist}", &metadata.artist.as_ref().map(|a| sanitize(a)).unwrap_or_else(|| "Unknown".to_string()))
            .replace("{album}", &metadata.album.as_ref().map(|a| sanitize(a)).unwrap_or_else(|| "Unknown".to_string()))
            .replace("{title}", &metadata.title.as_ref().map(|t| sanitize(t)).unwrap_or_else(|| "Unknown".to_string()))
            .replace("{year}", &metadata.year.map(|y| y.to_string()).unwrap_or_else(|| "Unknown".to_string()));

        let folder_path = base_folder.join(&expanded_pattern);
        
        fs::create_dir_all(&folder_path)
            .map_err(|e| format!("Failed to create folder structure: {}", e))?;

        let filename = path.file_name()
            .ok_or("Cannot determine filename")?;
        
        let new_path = folder_path.join(filename);

        if new_path.exists() {
            return Err(format!("File already exists at destination: {}", new_path.display()));
        }

        fs::rename(path, &new_path)
            .map_err(|e| format!("Failed to move file: {}", e))?;

        Ok(new_path)
    }

    pub fn rename_file(&self, path: &Path, metadata: &Metadata) -> Result<PathBuf, String> {
        let sanitize = |s: &str| -> String {
            s.chars()
                .map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' || c == '.' { c } else { '_' })
                .collect()
        };

        let artist = metadata.artist.as_ref()
            .map(|a| sanitize(a))
            .unwrap_or_else(|| "Unknown Artist".to_string());
        
        let title = metadata.title.as_ref()
            .map(|t| sanitize(t))
            .unwrap_or_else(|| "Unknown Title".to_string());

        let extension = path.extension()
            .and_then(|s| s.to_str())
            .ok_or("Cannot determine file extension")?;

        let new_filename = format!("{} - {}.{}", artist, title, extension);
        
        let new_path = path.parent()
            .ok_or("Cannot determine parent directory")?
            .join(&new_filename);

        if new_path.exists() {
            return Err(format!("File already exists: {}", new_path.display()));
        }

        fs::rename(path, &new_path)
            .map_err(|e| format!("Failed to rename file: {}", e))?;

        Ok(new_path)
    }

    pub fn restore_from_backup(&self, backup_path: &Path, original_path: &Path) -> Result<(), String> {
        let backup_data = fs::read_to_string(backup_path)
            .map_err(|e| format!("Failed to read backup file: {}", e))?;

        let metadata: Metadata = serde_json::from_str(&backup_data)
            .map_err(|e| format!("Failed to parse backup data: {}", e))?;

        self.write_metadata(original_path, &metadata)?;

        Ok(())
    }

    pub fn find_duplicates(&self, files: &[AudioFile]) -> Vec<Vec<usize>> {
        let mut duplicates: Vec<Vec<usize>> = Vec::new();
        let mut visited = vec![false; files.len()];

        for i in 0..files.len() {
            if visited[i] {
                continue;
            }

            let mut group = vec![i];
            let file_i = &files[i];

            if file_i.current_metadata.is_none() {
                continue;
            }

            let meta_i = file_i.current_metadata.as_ref().unwrap();

            for j in (i + 1)..files.len() {
                if visited[j] {
                    continue;
                }

                let file_j = &files[j];
                if let Some(meta_j) = &file_j.current_metadata {
                    if self.is_duplicate(meta_i, meta_j) {
                        group.push(j);
                        visited[j] = true;
                    }
                }
            }

            if group.len() > 1 {
                duplicates.push(group);
            }
        }

        duplicates
    }

    fn is_duplicate(&self, meta1: &Metadata, meta2: &Metadata) -> bool {
        let normalize = |s: Option<&String>| -> String {
            s.map(|s| s.to_lowercase().trim().to_string())
                .unwrap_or_default()
        };

        let artist_match = normalize(meta1.artist.as_ref()) == normalize(meta2.artist.as_ref())
            && !normalize(meta1.artist.as_ref()).is_empty();

        let title_match = normalize(meta1.title.as_ref()) == normalize(meta2.title.as_ref())
            && !normalize(meta1.title.as_ref()).is_empty();

        artist_match && title_match
    }
}
