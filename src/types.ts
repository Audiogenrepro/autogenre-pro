export interface AudioFile {
  path: string;
  filename: string;
  extension: string;
  current_metadata: Metadata | null;
}

export interface Metadata {
  title: string | null;
  artist: string | null;
  album: string | null;
  genre: string | null;
  year: number | null;
  bpm: number | null;
}

export interface MetadataResult {
  genre: string | null;
  artist: string | null;
  confidence: 'High' | 'Medium' | 'Low';
  source: string;
}

export interface EnhancedAudioFile extends AudioFile {
  suggested_metadata?: MetadataResult[];
  selected_genre?: string;
}

export interface AppSettings {
  spotify_client_id: string;
  spotify_client_secret: string;
  folder_pattern: string;
  backup_before_changes: boolean;
  organize_files: boolean;
  rename_files: boolean;
}
