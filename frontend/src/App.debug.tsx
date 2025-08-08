import React, { useState, useEffect } from 'react';

function App() {
  const [error, setError] = useState<string | null>(null);
  const [loaded, setLoaded] = useState(false);

  useEffect(() => {
    console.log('App component mounted');
    setLoaded(true);
  }, []);

  try {
    return (
      <div style={{ 
        padding: 20, 
        backgroundColor: '#1e1e1e', 
        color: '#cccccc', 
        height: '100vh',
        fontFamily: 'system-ui, -apple-system, sans-serif'
      }}>
        <h1>🐝 Hive Consensus - Debug Mode</h1>
        <p>Component loaded: {loaded ? 'Yes' : 'No'}</p>
        <p>Time: {new Date().toLocaleTimeString()}</p>
        
        <div style={{ marginTop: 20 }}>
          <h3>Testing Components:</h3>
          <ul>
            <li>React: ✅ Working</li>
            <li>State: {loaded ? '✅ Working' : '⏳ Loading'}</li>
            <li>Styles: ✅ Applied</li>
          </ul>
        </div>

        {error && (
          <div style={{ 
            marginTop: 20, 
            padding: 10, 
            backgroundColor: '#ff0000', 
            color: 'white' 
          }}>
            Error: {error}
          </div>
        )}
      </div>
    );
  } catch (err) {
    console.error('Render error:', err);
    return (
      <div style={{ color: 'red', padding: 20 }}>
        Render Error: {String(err)}
      </div>
    );
  }
}

export default App;