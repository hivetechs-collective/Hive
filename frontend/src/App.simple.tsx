import React from 'react';

function App() {
  return (
    <div style={{ 
      padding: 20, 
      backgroundColor: '#1e1e1e', 
      color: '#cccccc', 
      height: '100vh',
      fontFamily: 'system-ui, -apple-system, sans-serif'
    }}>
      <h1>🐝 Hive Consensus</h1>
      <p>Tauri app is working!</p>
      <button 
        onClick={() => console.log('Button clicked!')}
        style={{
          padding: '10px 20px',
          backgroundColor: '#007acc',
          color: 'white',
          border: 'none',
          borderRadius: 4,
          cursor: 'pointer',
          marginTop: 20
        }}
      >
        Test Button
      </button>
    </div>
  );
}

export default App;