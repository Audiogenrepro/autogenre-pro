import { EnhancedAudioFile } from '../types';

interface FileListProps {
  files: EnhancedAudioFile[];
}

export function FileList({ files }: FileListProps) {
  return (
    <div className="bg-gray-900 rounded-lg border border-gray-700 overflow-hidden">
      <table className="w-full">
        <thead className="bg-gray-800 border-b border-gray-700">
          <tr>
            <th className="px-4 py-3 text-left text-sm font-semibold">Filename</th>
            <th className="px-4 py-3 text-left text-sm font-semibold">Current Genre</th>
            <th className="px-4 py-3 text-left text-sm font-semibold">New Genre</th>
            <th className="px-4 py-3 text-left text-sm font-semibold">Confidence</th>
          </tr>
        </thead>
        <tbody>
          {files.length === 0 ? (
            <tr className="border-b border-gray-700">
              <td className="px-4 py-3 text-sm text-center text-gray-500" colSpan={4}>
                <div className="py-8">
                  No files scanned yet
                </div>
              </td>
            </tr>
          ) : (
            files.map((file, index) => (
              <tr key={index} className="border-b border-gray-700 hover:bg-gray-800 transition-colors">
                <td className="px-4 py-3 text-sm font-medium">{file.filename}</td>
                <td className="px-4 py-3 text-sm text-gray-400">
                  {file.current_metadata?.genre || 'N/A'}
                </td>
                <td className="px-4 py-3 text-sm text-gold-400 font-medium">
                  {file.suggested_metadata?.[0]?.genre || '-'}
                </td>
                <td className="px-4 py-3 text-sm">
                  {file.suggested_metadata?.[0] && (
                    <span
                      className={`px-2 py-1 rounded text-xs font-semibold ${
                        file.suggested_metadata[0].confidence === 'High'
                          ? 'bg-green-900 text-green-300'
                          : file.suggested_metadata[0].confidence === 'Medium'
                          ? 'bg-yellow-900 text-yellow-300'
                          : 'bg-red-900 text-red-300'
                      }`}
                    >
                      {file.suggested_metadata[0].confidence}
                    </span>
                  )}
                </td>
              </tr>
            ))
          )}
        </tbody>
      </table>
    </div>
  );
}
