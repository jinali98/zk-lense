import { useState, useEffect, useRef } from 'react';
import './App.css';


function App() {
  const [jsonData, setJsonData] = useState(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [source, setSource] = useState(null); // 'localhost' | 'file' | null
  const fileInputRef = useRef(null);

  // Check for port query parameter and fetch from localhost
  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    const port = params.get('port');
    
    if (port) {
      fetchFromLocalhost(port);
    }
  }, []);

  const fetchFromLocalhost = async (port) => {
    setLoading(true);
    setError(null);
    setSource('localhost');

    try {
      const response = await fetch(`http://localhost:${port}/data.json`);

      if (!response.ok) {
        throw new Error(`Server returned ${response.status}`);
      }

      const data = await response.json();
      setJsonData(data);
    } catch (err) {
      setError(
        `Failed to fetch from localhost:${port}. ` +
        `Make sure the CLI's local server is running.\n\n${err.message}`
      );
    } finally {
      setLoading(false);
    }
  };

  const handleFileSelect = (event) => {
    const file = event.target.files?.[0];
    if (!file) return;
    loadFile(file);
  };

  const handleDrop = (event) => {
    event.preventDefault();
    event.stopPropagation();
    const file = event.dataTransfer.files?.[0];
    if (file) {
      loadFile(file);
    }
  };

  const handleDragOver = (event) => {
    event.preventDefault();
    event.stopPropagation();
  };

  const loadFile = (file) => {
    setLoading(true);
    setError(null);
    setSource(file.name);

    const reader = new FileReader();
    reader.onload = (e) => {
      try {
        const data = JSON.parse(e.target.result);
        setJsonData(data);
      } catch (err) {
        setError(`Failed to parse JSON: ${err.message}`);
      } finally {
        setLoading(false);
      }
    };
    reader.onerror = () => {
      setError('Failed to read file');
      setLoading(false);
    };
    reader.readAsText(file);
  };

  const hasPortParam = new URLSearchParams(window.location.search).has('port');

  return (
    <div className="App" onDrop={handleDrop} onDragOver={handleDragOver}>
      <header className="App-header">
        <h1>zkprof Viewer</h1>

        {loading && (
          <div className="loading">
            <div className="spinner"></div>
            <p>Loading...</p>
          </div>
        )}

        {!loading && error && (
          <div className="error">
            <p>Error:</p>
            <pre>{error}</pre>
            <button onClick={() => fileInputRef.current?.click()}>
              Select File Manually
            </button>
          </div>
        )}

        {!loading && !jsonData && !error && !hasPortParam && (
          <div 
            className="drop-zone"
            onClick={() => fileInputRef.current?.click()}
          >
            <div className="drop-icon">📄</div>
            <p>Drop a JSON file here</p>
            <p className="hint">or click to select</p>
            <p className="hint cli-hint">
              Or run: <code>zkprof view report.json</code>
            </p>
          </div>
        )}

        <input
          ref={fileInputRef}
          type="file"
          accept=".json,application/json"
          onChange={handleFileSelect}
          style={{ display: 'none' }}
        />

        {jsonData && (
          <div className="json-viewer">
            <p className="file-path">📄 {source}</p>
            <pre className="json-content">
              {JSON.stringify(jsonData, null, 2)}
            </pre>
          </div>
        )}
      </header>
    </div>
  );
}

export default App;
