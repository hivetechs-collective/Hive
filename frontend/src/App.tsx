import React, { useState, useEffect } from 'react';
import { Layout, ConfigProvider, theme, Tabs, message } from 'antd';
import {
  FolderOutlined,
  CodeOutlined,
  ExperimentOutlined,
  BarChartOutlined,
  SettingOutlined,
  GithubOutlined,
} from '@ant-design/icons';
import { ConsensusPanel } from './components/ConsensusPanel';
import { FileExplorer } from './components/FileExplorer';
import { Terminal } from './components/Terminal';
import { StatusBar } from './components/StatusBar';
import { ActivityBar } from './components/ActivityBar';
import { ResizablePanels } from './components/ResizablePanels';
import { SettingsDialogEnhanced } from './components/SettingsDialogEnhanced';
import { useAppStore } from './stores/appStore';
import { useConsensusStore } from './stores/consensusStore';
import './App.css';

const { Content } = Layout;

function App() {
  const [activeView, setActiveView] = useState('explorer');
  const [activeTab, setActiveTab] = useState('consensus');
  const [showSettings, setShowSettings] = useState(false);
  const { darkMode, checkApiKeys } = useAppStore();
  
  useEffect(() => {
    // Check API key status on startup
    checkApiKeys().catch(err => {
      message.warning('No API keys configured. Please add your OpenRouter API key in settings.');
      setShowSettings(true); // Open settings if no API keys
    });
  }, []);

  const sidebarContent = () => {
    switch (activeView) {
      case 'explorer':
        return <FileExplorer />;
      case 'git':
        return <div style={{ padding: 20, color: 'var(--text-primary)' }}>Git Panel (Coming Soon)</div>;
      case 'analytics':
        return <div style={{ padding: 20, color: 'var(--text-primary)' }}>Analytics (Coming Soon)</div>;
      case 'settings':
        return <div style={{ padding: 20, color: 'var(--text-primary)' }}>Settings (use the gear icon)</div>;
      default:
        return null;
    }
  };
  
  // Handle settings button click
  const handleViewChange = (view: string) => {
    if (view === 'settings') {
      setShowSettings(true);
    } else {
      setActiveView(view);
    }
  };

  const mainContent = () => {
    const items = [
      {
        key: 'consensus',
        label: 'Consensus',
        children: <ConsensusPanel />,
      },
      {
        key: 'editor',
        label: 'Editor',
        children: <div style={{ padding: 20, color: 'var(--text-primary)' }}>Monaco Editor (Coming Soon)</div>,
      },
    ];

    return (
      <Tabs
        activeKey={activeTab}
        onChange={setActiveTab}
        items={items}
        style={{ height: '100%' }}
      />
    );
  };

  return (
    <ConfigProvider
      theme={{
        algorithm: theme.darkAlgorithm,
        token: {
          colorPrimary: '#007acc',
          borderRadius: 0,
        },
        components: {
          Layout: {
            bodyBg: '#1e1e1e',
            headerBg: '#252526',
            siderBg: '#252526',
            footerBg: '#007acc',
          },
        },
      }}
    >
      <div className="app-container">
        <ActivityBar activeView={activeView} onViewChange={handleViewChange} />
        
        <Layout className="main-layout">
          <ResizablePanels
            panels={[
              {
                id: 'sidebar',
                defaultSize: 240,
                minSize: 150,
                maxSize: 400,
                content: (
                  <div className="sidebar">
                    <div className="app-logo">
                      <h3>🐝 Hive Consensus</h3>
                    </div>
                    {sidebarContent()}
                  </div>
                ),
              },
              {
                id: 'editor',
                defaultSize: 'flex',
                content: (
                  <ResizablePanels
                    direction="vertical"
                    panels={[
                      {
                        id: 'main',
                        defaultSize: 'flex',
                        content: mainContent(),
                      },
                      {
                        id: 'terminal',
                        defaultSize: 300,
                        minSize: 100,
                        maxSize: 500,
                        content: <Terminal />,
                      },
                    ]}
                  />
                ),
              },
            ]}
          />
          
          <StatusBar />
        </Layout>
        
        <SettingsDialogEnhanced 
          visible={showSettings}
          onClose={() => setShowSettings(false)}
        />
      </div>
    </ConfigProvider>
  );
}

export default App;