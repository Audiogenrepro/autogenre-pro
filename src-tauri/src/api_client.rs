use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::Mutex;

static SPOTIFY_TOKEN_CACHE: Mutex<Option<TokenCache>> = Mutex::new(None);
static BEATPORT_TOKEN_CACHE: Mutex<Option<TokenCache>> = Mutex::new(None);

#[derive(Debug, Clone)]
struct TokenCache {
    access_token: String,
    expires_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataResult {
    pub genre: Option<String>,
    pub artist: Option<String>,
    pub confidence: Confidence,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Confidence {
    High,
    Medium,
    Low,
}

#[derive(Debug, Deserialize)]
struct SpotifyTokenResponse {
    access_token: String,
}

#[derive(Debug, Deserialize)]
struct SpotifySearchResponse {
    tracks: SpotifyTracks,
}

#[derive(Debug, Deserialize)]
struct SpotifyTracks {
    items: Vec<SpotifyTrack>,
}

#[derive(Debug, Deserialize)]
struct SpotifyTrack {
    artists: Vec<SpotifyArtist>,
    id: String,
}

#[derive(Debug, Deserialize)]
struct SpotifyArtist {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct SpotifyArtistDetails {
    genres: Vec<String>,
}

pub struct SpotifyClient {
    client_id: Option<String>,
    client_secret: Option<String>,
}

impl SpotifyClient {
    pub fn new(client_id: Option<String>, client_secret: Option<String>) -> Self {
        SpotifyClient {
            client_id,
            client_secret,
        }
    }

    async fn get_access_token(&self) -> Result<String, String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        {
            let cache = SPOTIFY_TOKEN_CACHE.lock().unwrap();
            if let Some(cached) = cache.as_ref() {
                if cached.expires_at > now {
                    return Ok(cached.access_token.clone());
                }
            }
        }

        let client_id = self.client_id.as_ref()
            .ok_or("Spotify client ID not configured")?;
        let client_secret = self.client_secret.as_ref()
            .ok_or("Spotify client secret not configured")?;

        let client = Client::new();
        let mut params = HashMap::new();
        params.insert("grant_type", "client_credentials");

        let response = client
            .post("https://accounts.spotify.com/api/token")
            .basic_auth(client_id, Some(client_secret))
            .form(&params)
            .send()
            .await
            .map_err(|e| format!("Failed to request token: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Spotify auth failed: {}", response.status()));
        }

        let token_response: SpotifyTokenResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse token response: {}", e))?;

        let expires_at = now + 3000;
        
        {
            let mut cache = SPOTIFY_TOKEN_CACHE.lock().unwrap();
            *cache = Some(TokenCache {
                access_token: token_response.access_token.clone(),
                expires_at,
            });
        }

        Ok(token_response.access_token)
    }

    pub async fn search_track(
        &self,
        artist: &str,
        title: &str,
    ) -> Result<MetadataResult, String> {
        if self.client_id.is_none() || self.client_secret.is_none() {
            return Err("Spotify API credentials not configured".to_string());
        }

        let access_token = self.get_access_token().await?;
        let client = Client::new();

        let quote_if_multiword = |s: &str| {
            if s.contains(' ') {
                format!("\"{}\"", s)
            } else {
                s.to_string()
            }
        };

        let query = format!("artist:{} track:{}", quote_if_multiword(artist), quote_if_multiword(title));
        let response = client
            .get("https://api.spotify.com/v1/search")
            .bearer_auth(&access_token)
            .query(&[("q", query.as_str()), ("type", "track"), ("limit", "1")])
            .send()
            .await
            .map_err(|e| format!("Spotify search failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Spotify API error: {}", response.status()));
        }

        let search_response: SpotifySearchResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse search response: {}", e))?;

        if search_response.tracks.items.is_empty() {
            return Ok(MetadataResult {
                genre: None,
                artist: Some(artist.to_string()),
                confidence: Confidence::Low,
                source: "Spotify (No match)".to_string(),
            });
        }

        let track = &search_response.tracks.items[0];
        let artist_id = &track.artists[0].id;
        let artist_name = &track.artists[0].name;

        let artist_response = client
            .get(format!("https://api.spotify.com/v1/artists/{}", artist_id))
            .bearer_auth(&access_token)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch artist details: {}", e))?;

        if !artist_response.status().is_success() {
            return Ok(MetadataResult {
                genre: None,
                artist: Some(artist_name.clone()),
                confidence: Confidence::Medium,
                source: "Spotify".to_string(),
            });
        }

        let artist_details: SpotifyArtistDetails = artist_response
            .json()
            .await
            .map_err(|e| format!("Failed to parse artist details: {}", e))?;

        let genre = artist_details.genres.first().cloned();
        let confidence = if genre.is_some() {
            Confidence::High
        } else {
            Confidence::Medium
        };

        Ok(MetadataResult {
            genre,
            artist: Some(artist_name.clone()),
            confidence,
            source: "Spotify".to_string(),
        })
    }
}

#[derive(Debug, Deserialize)]
struct MusicBrainzSearchResponse {
    recordings: Vec<MusicBrainzRecording>,
}

#[derive(Debug, Deserialize)]
struct MusicBrainzRecording {
    #[serde(rename = "artist-credit")]
    artist_credit: Vec<MusicBrainzArtistCredit>,
    tags: Option<Vec<MusicBrainzTag>>,
    genres: Option<Vec<MusicBrainzGenre>>,
}

#[derive(Debug, Deserialize)]
struct MusicBrainzArtistCredit {
    name: String,
}

#[derive(Debug, Deserialize)]
struct MusicBrainzTag {
    name: String,
}

#[derive(Debug, Deserialize)]
struct MusicBrainzGenre {
    name: String,
}

pub struct MusicBrainzClient {
    base_url: String,
}

impl MusicBrainzClient {
    pub fn new() -> Self {
        MusicBrainzClient {
            base_url: "https://musicbrainz.org/ws/2".to_string(),
        }
    }

    pub async fn search_track(
        &self,
        artist: &str,
        title: &str,
    ) -> Result<MetadataResult, String> {
        let client = Client::new();
        
        let query = format!("artist:{} AND recording:{}", artist, title);
        let response = client
            .get(format!("{}/recording", self.base_url))
            .query(&[("query", query.as_str()), ("fmt", "json"), ("limit", "1"), ("inc", "tags+genres")])
            .header("User-Agent", "AutoGenrePro/0.1.0 ( contact@example.com )")
            .send()
            .await
            .map_err(|e| format!("MusicBrainz search failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("MusicBrainz API error: {}", response.status()));
        }

        let search_response: MusicBrainzSearchResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse MusicBrainz response: {}", e))?;

        if search_response.recordings.is_empty() {
            return Ok(MetadataResult {
                genre: None,
                artist: Some(artist.to_string()),
                confidence: Confidence::Low,
                source: "MusicBrainz (No match)".to_string(),
            });
        }

        let recording = &search_response.recordings[0];
        
        let artist_name = recording.artist_credit
            .first()
            .map(|ac| ac.name.clone())
            .unwrap_or_else(|| artist.to_string());
        
        let genre = recording.genres.as_ref()
            .and_then(|genres| genres.first())
            .map(|genre| genre.name.clone())
            .or_else(|| {
                recording.tags.as_ref()
                    .and_then(|tags| tags.first())
                    .map(|tag| tag.name.clone())
            });

        let confidence = if genre.is_some() {
            Confidence::Medium
        } else {
            Confidence::Low
        };

        Ok(MetadataResult {
            genre,
            artist: Some(artist_name),
            confidence,
            source: "MusicBrainz".to_string(),
        })
    }
}

#[derive(Debug, Deserialize)]
struct BeatportTokenResponse {
    access_token: String,
    expires_in: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct BeatportSearchResponse {
    results: Vec<BeatportTrack>,
}

#[derive(Debug, Deserialize)]
struct BeatportTrack {
    #[serde(default)]
    genre: Option<BeatportGenre>,
    #[serde(default)]
    sub_genre: Option<BeatportGenre>,
    artists: Vec<BeatportArtist>,
}

#[derive(Debug, Deserialize)]
struct BeatportGenre {
    name: String,
}

#[derive(Debug, Deserialize)]
struct BeatportArtist {
    name: String,
}

pub struct BeatportClient {
    username: Option<String>,
    password: Option<String>,
}

impl BeatportClient {
    pub fn new(username: Option<String>, password: Option<String>) -> Self {
        BeatportClient {
            username,
            password,
        }
    }

    async fn get_access_token(&self) -> Result<String, String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        {
            let cache = BEATPORT_TOKEN_CACHE.lock().unwrap();
            if let Some(cached) = cache.as_ref() {
                if cached.expires_at > now {
                    return Ok(cached.access_token.clone());
                }
            }
        }

        let username = self.username.as_ref()
            .ok_or("Beatport username not configured")?;
        let password = self.password.as_ref()
            .ok_or("Beatport password not configured")?;

        let client = Client::new();
        
        let client_id = "oeGScrHHsv1K1vO2Mby3sHQ7oZNWpViH";
        
        let mut params = HashMap::new();
        params.insert("grant_type", "password");
        params.insert("client_id", client_id);
        params.insert("username", username.as_str());
        params.insert("password", password.as_str());

        let response = client
            .post("https://api.beatport.com/v4/auth/o/token/")
            .form(&params)
            .send()
            .await
            .map_err(|e| format!("Failed to request Beatport token: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("Beatport auth failed ({}): {}", status, body));
        }

        let token_response: BeatportTokenResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Beatport token response: {}", e))?;

        let expires_in = token_response.expires_in.unwrap_or(3600);
        let expires_at = now + expires_in - 300;
        
        {
            let mut cache = BEATPORT_TOKEN_CACHE.lock().unwrap();
            *cache = Some(TokenCache {
                access_token: token_response.access_token.clone(),
                expires_at,
            });
        }

        Ok(token_response.access_token)
    }

    pub async fn search_track(
        &self,
        artist: &str,
        title: &str,
    ) -> Result<MetadataResult, String> {
        if self.username.is_none() || self.password.is_none() {
            return Err("Beatport credentials not configured".to_string());
        }

        let access_token = self.get_access_token().await?;
        let client = Client::new();

        let query = format!("{} {}", artist, title);
        
        let response = client
            .get("https://api.beatport.com/v4/catalog/tracks/")
            .bearer_auth(&access_token)
            .query(&[("q", query.as_str()), ("per_page", "1")])
            .send()
            .await
            .map_err(|e| format!("Beatport search failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Beatport API error: {}", response.status()));
        }

        let search_response: BeatportSearchResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Beatport response: {}", e))?;

        if search_response.results.is_empty() {
            return Ok(MetadataResult {
                genre: None,
                artist: Some(artist.to_string()),
                confidence: Confidence::Low,
                source: "Beatport (No match)".to_string(),
            });
        }

        let track = &search_response.results[0];
        let artist_name = track.artists
            .first()
            .map(|a| a.name.clone())
            .unwrap_or_else(|| artist.to_string());

        let genre = track.sub_genre.as_ref()
            .or(track.genre.as_ref())
            .map(|g| g.name.clone());

        let confidence = if track.sub_genre.is_some() {
            Confidence::High
        } else if track.genre.is_some() {
            Confidence::High
        } else {
            Confidence::Low
        };

        Ok(MetadataResult {
            genre,
            artist: Some(artist_name),
            confidence,
            source: "Beatport".to_string(),
        })
    }
}
