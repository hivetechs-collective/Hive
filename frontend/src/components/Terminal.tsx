import React, { useEffect, useRef, useState } from 'react';
import { Tabs, Button, Space } from 'antd';
import { PlusOutlined, CloseOutlined, ClearOutlined } from '@ant-design/icons';
import { Terminal as XTerm } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { WebLinksAddon } from '@xterm/addon-web-links';
import { invoke } from '@tauri-apps/api/core';
import '@xterm/xterm/css/xterm.css';

interface TerminalTab {
  id: string;
  title: string;
  terminal: XTerm;
  fitAddon: FitAddon;
}

export const Terminal: React.FC = () => {
  const [tabs, setTabs] = useState<TerminalTab[]>([]);
  const [activeKey, setActiveKey] = useState<string>('');
  const terminalRefs = useRef<{ [key: string]: HTMLDivElement }>({});

  useEffect(() => {
    // Create initial terminal
    createNewTerminal();
  }, []);

  const createNewTerminal = () => {
    const id = `terminal-${Date.now()}`;
    const terminal = new XTerm({
      cursorBlink: true,
      fontSize: 14,
      fontFamily: 'Menlo, Monaco, "Courier New", monospace',
      theme: {
        background: '#1e1e1e',
        foreground: '#cccccc',
        cursor: '#ffffff',
        black: '#000000',
        red: '#cd3131',
        green: '#0dbc79',
        yellow: '#e5e510',
        blue: '#2472c8',
        magenta: '#bc3fbc',
        cyan: '#11a8cd',
        white: '#e5e5e5',
        brightBlack: '#666666',
        brightRed: '#f14c4c',
        brightGreen: '#23d18b',
        brightYellow: '#f5f543',
        brightBlue: '#3b8eea',
        brightMagenta: '#d670d6',
        brightCyan: '#29b8db',
        brightWhite: '#e5e5e5',
      },
    });

    const fitAddon = new FitAddon();
    terminal.loadAddon(fitAddon);
    terminal.loadAddon(new WebLinksAddon());

    const newTab: TerminalTab = {
      id,
      title: `Terminal ${tabs.length + 1}`,
      terminal,
      fitAddon,
    };

    setTabs(prev => [...prev, newTab]);
    setActiveKey(id);

    // Connect to backend terminal
    setTimeout(() => {
      if (terminalRefs.current[id]) {
        terminal.open(terminalRefs.current[id]);
        fitAddon.fit();
        
        // For now, just show a welcome message
        // In production, this would connect to the PTY backend
        terminal.writeln('Welcome to Hive Consensus Terminal');
        terminal.writeln('');
        terminal.write('$ ');
        
        // Handle input
        terminal.onData((data) => {
          // In production, send to PTY backend
          // For now, just echo
          if (data === '\r') {
            terminal.write('\r\n$ ');
          } else if (data === '\u007F') { // Backspace
            terminal.write('\b \b');
          } else {
            terminal.write(data);
          }
        });
      }
    }, 0);
  };

  const closeTerminal = (targetKey: string) => {
    const newTabs = tabs.filter(tab => tab.id !== targetKey);
    setTabs(newTabs);
    
    if (targetKey === activeKey && newTabs.length > 0) {
      setActiveKey(newTabs[newTabs.length - 1].id);
    }
    
    // Dispose terminal
    const tab = tabs.find(t => t.id === targetKey);
    if (tab) {
      tab.terminal.dispose();
    }
  };

  const clearTerminal = () => {
    const activeTab = tabs.find(tab => tab.id === activeKey);
    if (activeTab) {
      activeTab.terminal.clear();
    }
  };

  useEffect(() => {
    // Handle resize
    const handleResize = () => {
      tabs.forEach(tab => {
        if (tab.fitAddon) {
          tab.fitAddon.fit();
        }
      });
    };

    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, [tabs]);

  const tabItems = tabs.map(tab => ({
    key: tab.id,
    label: (
      <span>
        {tab.title}
        {tabs.length > 1 && (
          <CloseOutlined
            style={{ marginLeft: 8, fontSize: 12 }}
            onClick={(e) => {
              e.stopPropagation();
              closeTerminal(tab.id);
            }}
          />
        )}
      </span>
    ),
    children: (
      <div
        ref={el => {
          if (el) terminalRefs.current[tab.id] = el;
        }}
        style={{ height: 'calc(100% - 40px)', width: '100%' }}
      />
    ),
  }));

  return (
    <div className="terminal-container">
      <Tabs
        type="editable-card"
        activeKey={activeKey}
        onChange={setActiveKey}
        items={tabItems}
        addIcon={<PlusOutlined />}
        onEdit={(targetKey, action) => {
          if (action === 'add') {
            createNewTerminal();
          }
        }}
        tabBarExtraContent={
          <Space>
            <Button
              size="small"
              icon={<ClearOutlined />}
              onClick={clearTerminal}
              title="Clear Terminal"
            />
          </Space>
        }
        style={{ height: '100%' }}
      />
    </div>
  );
};