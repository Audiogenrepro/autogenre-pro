import { useState, useEffect } from "react";
import { Folder, Play, StopCircle, Save, Settings as SettingsIcon, Music } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { EnhancedAudioFile, AppSettings } from "./types";
import { FileList } from "./components/FileList";
import { SettingsModal } from "./components/SettingsModal";

function App() {
  const [selectedFolder, setSelectedFolder] = useState<string>("");
  const [isScanning, setIsScanning] = useState(false);
  const [files, setFiles] = useState<EnhancedAudioFile[]>([]);
  const [progress, setProgress] = useState(0);
  const [statusMessage, setStatusMessage] = useState("Ready to scan");
  const [showSettings, setShowSettings] = useState(false);
  const [settings, setSettings] = useState<AppSettings>({
    spotify_client_id: '',
    spotify_client_secret: '',
    folder_pattern: '{genre}',
    backup_before_changes: true,
    organize_files: false,
    rename_files: false,
  });

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      const loadedSettings = await invoke<AppSettings>('load_settings');
      setSettings(loadedSettings);
    } catch (error) {
      console.error("Error loading settings:", error);
    }
  };

  const handleSelectFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
      });

      if (selected && typeof selected === 'string') {
        setSelectedFolder(selected);
        setStatusMessage(`Selected: ${selected}`);
      }
    } catch (error) {
      console.error("Error selecting folder:", error);
      setStatusMessage("Error selecting folder");
    }
  };

  const handleStartScan = async () => {
    if (!selectedFolder) {
      setStatusMessage("Please select a folder first");
      return;
    }

    setIsScanning(true);
    setStatusMessage("Scanning for audio files...");
    setProgress(10);

    try {
      const scannedFiles = await invoke<EnhancedAudioFile[]>("scan_folder", {
        path: selectedFolder,
      });

      setFiles(scannedFiles);
      setProgress(50);
      setStatusMessage(`Found ${scannedFiles.length} audio files. Fetching metadata...`);

      for (let i = 0; i < scannedFiles.length; i++) {
        const file = scannedFiles[i];
        if (file.current_metadata?.artist && file.current_metadata?.title) {
          try {
            const metadata = await invoke("fetch_metadata", {
              artist: file.current_metadata.artist,
              title: file.current_metadata.title,
            });
            file.suggested_metadata = metadata as any;
          } catch (error) {
            console.error(`Error fetching metadata for ${file.filename}:`, error);
          }
        }
        setProgress(50 + ((i + 1) / scannedFiles.length) * 50);
      }

      setFiles([...scannedFiles]);
      setProgress(100);
      setStatusMessage(`Completed! Processed ${scannedFiles.length} files`);
    } catch (error) {
      console.error("Error scanning folder:", error);
      setStatusMessage(`Error: ${error}`);
    } finally {
      setIsScanning(false);
    }
  };

  const handleApplyChanges = async () => {
    setStatusMessage("Applying metadata changes...");
    setProgress(0);
    let successCount = 0;
    let errorCount = 0;
    let organizedCount = 0;
    const errors: string[] = [];
    const updatedFiles: EnhancedAudioFile[] = [...files];

    for (let i = 0; i < files.length; i++) {
      const file = files[i];
      
      if (file.suggested_metadata && file.suggested_metadata.length > 0) {
        const suggestedGenre = file.suggested_metadata[0].genre;
        const suggestedArtist = file.suggested_metadata[0].artist;
        
        if (suggestedGenre || suggestedArtist) {
          try {
            const updatedMetadata = {
              title: file.current_metadata?.title || null,
              artist: suggestedArtist || file.current_metadata?.artist || null,
              album: file.current_metadata?.album || null,
              genre: suggestedGenre || file.current_metadata?.genre || null,
              year: file.current_metadata?.year || null,
              bpm: file.current_metadata?.bpm || null,
            };

            await invoke("update_metadata", {
              file_path: file.path,
              metadata: updatedMetadata,
              backup: settings.backup_before_changes,
            });

            updatedFiles[i].current_metadata = updatedMetadata;
            
            let currentPath = file.path;

            if (settings.rename_files) {
              try {
                const newPath = await invoke<string>("rename_file", {
                  file_path: currentPath,
                  metadata: updatedMetadata,
                });
                currentPath = newPath;
                updatedFiles[i].path = newPath;
                updatedFiles[i].filename = newPath.split('/').pop() || updatedFiles[i].filename;
              } catch (renError) {
                console.error(`Error renaming ${file.filename}:`, renError);
              }
            }
            
            if (settings.organize_files && selectedFolder) {
              try {
                const newPath = await invoke<string>("organize_files", {
                  file_path: currentPath,
                  metadata: updatedMetadata,
                  base_folder: selectedFolder,
                });
                updatedFiles[i].path = newPath;
                organizedCount++;
              } catch (orgError) {
                console.error(`Error organizing ${file.filename}:`, orgError);
              }
            }
            
            successCount++;
          } catch (error) {
            const errorMsg = error instanceof Error ? error.message : String(error);
            console.error(`Error updating ${file.filename}:`, errorMsg);
            errors.push(`${file.filename}: ${errorMsg}`);
            errorCount++;
          }
        }
      }
      
      setProgress(((i + 1) / files.length) * 100);
    }

    setFiles(updatedFiles);

    if (errors.length > 0) {
      console.error("Metadata update errors:", errors);
    }

    let message = `Completed! ${successCount} files updated successfully`;
    if (organizedCount > 0) {
      message += `, ${organizedCount} organized`;
    }
    if (errorCount > 0) {
      message += `, ${errorCount} failed`;
    }
    
    setStatusMessage(message);
  };

  return (
    <div className="min-h-screen bg-gray-900 text-gray-100">
      <div className="flex flex-col h-screen">
        <header className="bg-gray-800 border-b border-gray-700 px-6 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <Music className="w-8 h-8 text-gold-400" />
              <h1 className="text-2xl font-bold">AutoGenre Pro</h1>
            </div>
            <div className="flex gap-3">
              <button
                onClick={handleSelectFolder}
                className="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded-lg flex items-center gap-2 transition-colors"
              >
                <Folder className="w-4 h-4" />
                Select Folder
              </button>
              <button
                onClick={isScanning ? () => setIsScanning(false) : handleStartScan}
                disabled={!selectedFolder}
                className="px-4 py-2 bg-gold-500 hover:bg-gold-400 text-gray-900 font-semibold rounded-lg flex items-center gap-2 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isScanning ? (
                  <>
                    <StopCircle className="w-4 h-4" />
                    Stop Scan
                  </>
                ) : (
                  <>
                    <Play className="w-4 h-4" />
                    Start Scan
                  </>
                )}
              </button>
              <button
                onClick={() => setShowSettings(true)}
                className="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded-lg flex items-center gap-2 transition-colors"
              >
                <SettingsIcon className="w-4 h-4" />
                Settings
              </button>
            </div>
          </div>
        </header>

        <main className="flex-1 flex overflow-hidden">
          <div className="flex-1 flex flex-col">
            <div className="flex-1 overflow-auto p-6">
              <div className="bg-gray-800 rounded-lg border border-gray-700 h-full flex flex-col">
                {!selectedFolder ? (
                  <div className="flex-1 flex items-center justify-center">
                    <div className="text-center text-gray-400">
                      <Folder className="w-16 h-16 mx-auto mb-4 opacity-50" />
                      <p className="text-lg">Select a folder to start scanning</p>
                      <p className="text-sm mt-2">Click "Select Folder" to choose your music library</p>
                    </div>
                  </div>
                ) : (
                  <div className="flex-1 flex flex-col p-6">
                    <div className="mb-4">
                      <h2 className="text-xl font-semibold mb-2">File List</h2>
                      <p className="text-sm text-gray-400">Folder: {selectedFolder}</p>
                    </div>
                    <FileList files={files} />
                  </div>
                )}
              </div>
            </div>

            <footer className="bg-gray-800 border-t border-gray-700 px-6 py-4">
              <div className="flex items-center justify-between">
                <div className="flex-1 mr-6">
                  <div className="w-full bg-gray-700 rounded-full h-2 overflow-hidden">
                    <div
                      className="bg-gold-500 h-full transition-all duration-300"
                      style={{ width: `${progress}%` }}
                    ></div>
                  </div>
                  <p className="text-xs text-gray-400 mt-2">{statusMessage}</p>
                </div>
                <button
                  onClick={handleApplyChanges}
                  disabled={files.length === 0 || isScanning}
                  className="px-4 py-2 bg-green-600 hover:bg-green-500 rounded-lg flex items-center gap-2 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  <Save className="w-4 h-4" />
                  Apply Changes
                </button>
              </div>
            </footer>
          </div>
        </main>
      </div>

      <SettingsModal 
        isOpen={showSettings} 
        onClose={() => setShowSettings(false)}
        onSettingsSaved={() => {
          loadSettings();
          setStatusMessage("Settings saved successfully!");
        }}
      />
    </div>
  );
}

export default App;
