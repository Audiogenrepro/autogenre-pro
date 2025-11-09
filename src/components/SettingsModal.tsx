import { useState, useEffect } from 'react';
import { X, AlertCircle, CheckCircle } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { AppSettings } from '../types';

interface SettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSettingsSaved: () => void;
}

export function SettingsModal({ isOpen, onClose, onSettingsSaved }: SettingsModalProps) {
  const [settings, setSettings] = useState<AppSettings>({
    spotify_client_id: '',
    spotify_client_secret: '',
    folder_pattern: '{genre}',
    backup_before_changes: true,
    organize_files: false,
    rename_files: false,
  });
  const [saveStatus, setSaveStatus] = useState<'idle' | 'saving' | 'success' | 'error'>('idle');
  const [errorMessage, setErrorMessage] = useState('');

  useEffect(() => {
    if (isOpen) {
      loadSettings();
    }
  }, [isOpen]);

  const loadSettings = async () => {
    try {
      const loadedSettings = await invoke<AppSettings>('load_settings');
      setSettings(loadedSettings);
    } catch (error) {
      console.error('Error loading settings:', error);
    }
  };

  const handleSave = async () => {
    setSaveStatus('saving');
    setErrorMessage('');
    
    try {
      await invoke('save_settings', { settings });
      setSaveStatus('success');
      onSettingsSaved();
      setTimeout(() => {
        onClose();
        setSaveStatus('idle');
      }, 1000);
    } catch (error) {
      setSaveStatus('error');
      setErrorMessage(error as string);
      setTimeout(() => setSaveStatus('idle'), 3000);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-gray-800 rounded-lg shadow-xl w-full max-w-2xl mx-4">
        <div className="flex items-center justify-between p-6 border-b border-gray-700">
          <h2 className="text-xl font-bold">Settings</h2>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-200 transition-colors"
          >
            <X className="w-6 h-6" />
          </button>
        </div>
        
        <div className="p-6 space-y-6">
          <div>
            <h3 className="text-lg font-semibold mb-4">API Configuration</h3>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium mb-2">
                  Spotify Client ID
                </label>
                <input
                  type="text"
                  value={settings.spotify_client_id}
                  onChange={(e) => setSettings({ ...settings, spotify_client_id: e.target.value })}
                  className="w-full px-4 py-2 bg-gray-900 border border-gray-700 rounded-lg focus:outline-none focus:ring-2 focus:ring-gold-500"
                  placeholder="Enter your Spotify Client ID"
                />
                <p className="text-xs text-gray-400 mt-1">
                  Get your credentials from <a href="https://developer.spotify.com/dashboard" target="_blank" rel="noopener noreferrer" className="text-gold-400 hover:underline">Spotify Developer Dashboard</a>
                </p>
              </div>
              <div>
                <label className="block text-sm font-medium mb-2">
                  Spotify Client Secret
                </label>
                <input
                  type="password"
                  value={settings.spotify_client_secret}
                  onChange={(e) => setSettings({ ...settings, spotify_client_secret: e.target.value })}
                  className="w-full px-4 py-2 bg-gray-900 border border-gray-700 rounded-lg focus:outline-none focus:ring-2 focus:ring-gold-500"
                  placeholder="Enter your Spotify Client Secret"
                />
              </div>
            </div>
            <div className="mt-4 p-4 bg-blue-900 bg-opacity-30 border border-blue-700 rounded-lg">
              <p className="text-sm text-blue-200">
                <strong>Beatport Integration (Advanced):</strong> For security, Beatport credentials must be set as environment variables before launching the app:
                <code className="block mt-2 p-2 bg-gray-900 rounded text-xs">
                  BEATPORT_USERNAME=your_username<br/>
                  BEATPORT_PASSWORD=your_password
                </code>
                <span className="block mt-2 text-xs">
                  This provides enhanced genre detection for electronic music. If not configured, Beatport will be skipped.
                </span>
              </p>
            </div>
          </div>

          <div>
            <h3 className="text-lg font-semibold mb-4">File Organization</h3>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium mb-2">
                  Folder Pattern
                </label>
                <input
                  type="text"
                  value={settings.folder_pattern}
                  onChange={(e) => setSettings({ ...settings, folder_pattern: e.target.value })}
                  className="w-full px-4 py-2 bg-gray-900 border border-gray-700 rounded-lg focus:outline-none focus:ring-2 focus:ring-gold-500"
                  placeholder="e.g., {genre}/{artist}/{title}"
                />
                <p className="text-xs text-gray-400 mt-2">
                  Available placeholders: {'{genre}'}, {'{artist}'}, {'{title}'}, {'{album}'}, {'{year}'}
                </p>
              </div>
              <div className="flex items-center gap-2">
                <input
                  type="checkbox"
                  id="backup"
                  checked={settings.backup_before_changes}
                  onChange={(e) => setSettings({ ...settings, backup_before_changes: e.target.checked })}
                  className="w-4 h-4 text-gold-500 bg-gray-900 border-gray-700 rounded focus:ring-gold-500"
                />
                <label htmlFor="backup" className="text-sm">
                  Create JSON backup before applying changes
                </label>
              </div>
              <div className="flex items-center gap-2">
                <input
                  type="checkbox"
                  id="organize"
                  checked={settings.organize_files}
                  onChange={(e) => setSettings({ ...settings, organize_files: e.target.checked })}
                  className="w-4 h-4 text-gold-500 bg-gray-900 border-gray-700 rounded focus:ring-gold-500"
                />
                <label htmlFor="organize" className="text-sm">
                  Automatically organize files into folders
                </label>
              </div>
              <div className="flex items-center gap-2">
                <input
                  type="checkbox"
                  id="rename"
                  checked={settings.rename_files}
                  onChange={(e) => setSettings({ ...settings, rename_files: e.target.checked })}
                  className="w-4 h-4 text-gold-500 bg-gray-900 border-gray-700 rounded focus:ring-gold-500"
                />
                <label htmlFor="rename" className="text-sm">
                  Rename files to "Artist - Song Name" format
                </label>
              </div>
            </div>
          </div>

          {saveStatus === 'error' && (
            <div className="flex items-center gap-2 p-3 bg-red-900 bg-opacity-30 border border-red-700 rounded-lg">
              <AlertCircle className="w-5 h-5 text-red-500" />
              <p className="text-sm text-red-300">{errorMessage || 'Failed to save settings'}</p>
            </div>
          )}

          {saveStatus === 'success' && (
            <div className="flex items-center gap-2 p-3 bg-green-900 bg-opacity-30 border border-green-700 rounded-lg">
              <CheckCircle className="w-5 h-5 text-green-500" />
              <p className="text-sm text-green-300">Settings saved successfully!</p>
            </div>
          )}
        </div>

        <div className="flex justify-end gap-3 p-6 border-t border-gray-700">
          <button
            onClick={onClose}
            className="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded-lg transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={handleSave}
            disabled={saveStatus === 'saving'}
            className="px-4 py-2 bg-gold-500 hover:bg-gold-400 text-gray-900 font-semibold rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {saveStatus === 'saving' ? 'Saving...' : 'Save Settings'}
          </button>
        </div>
      </div>
    </div>
  );
}
