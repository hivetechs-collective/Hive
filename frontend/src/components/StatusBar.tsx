import React, { useState, useEffect } from 'react';
import { Space } from 'antd';
import {
  BranchesOutlined,
  SyncOutlined,
  CheckCircleOutlined,
  ExclamationCircleOutlined,
  InfoCircleOutlined,
} from '@ant-design/icons';
import { invoke } from '@tauri-apps/api/core';

export const StatusBar: React.FC = () => {
  const [gitBranch, setGitBranch] = useState('main');
  const [gitStatus, setGitStatus] = useState({ modified: 0, added: 0, deleted: 0 });
  const [consensusStatus, setConsensusStatus] = useState<'idle' | 'running' | 'error'>('idle');
  const [currentFile, setCurrentFile] = useState('');
  const [position, setPosition] = useState({ line: 1, column: 1 });
  const [problems, setProblems] = useState({ errors: 0, warnings: 0 });

  useEffect(() => {
    // Poll for git status
    const interval = setInterval(async () => {
      // In production, call git status command
      // For now, use mock data
      setGitBranch('main');
      setGitStatus({ modified: 2, added: 1, deleted: 0 });
    }, 5000);

    return () => clearInterval(interval);
  }, []);

  const getStatusIcon = () => {
    switch (consensusStatus) {
      case 'running':
        return <SyncOutlined spin style={{ color: '#007acc' }} />;
      case 'error':
        return <ExclamationCircleOutlined style={{ color: '#f44336' }} />;
      default:
        return <CheckCircleOutlined style={{ color: '#4caf50' }} />;
    }
  };

  return (
    <div className="status-bar">
      <div style={{ display: 'flex', gap: 15 }}>
        {/* Git Status */}
        <div className="status-bar-item">
          <BranchesOutlined />
          <span>{gitBranch}</span>
          {(gitStatus.modified > 0 || gitStatus.added > 0 || gitStatus.deleted > 0) && (
            <span style={{ marginLeft: 5 }}>
              {gitStatus.modified > 0 && <span>~{gitStatus.modified}</span>}
              {gitStatus.added > 0 && <span style={{ marginLeft: 3 }}>+{gitStatus.added}</span>}
              {gitStatus.deleted > 0 && <span style={{ marginLeft: 3 }}>-{gitStatus.deleted}</span>}
            </span>
          )}
        </div>

        {/* Problems */}
        {(problems.errors > 0 || problems.warnings > 0) && (
          <div className="status-bar-item">
            {problems.errors > 0 && (
              <>
                <ExclamationCircleOutlined style={{ color: '#f44336' }} />
                <span>{problems.errors}</span>
              </>
            )}
            {problems.warnings > 0 && (
              <>
                <InfoCircleOutlined style={{ color: '#ff9800', marginLeft: problems.errors > 0 ? 8 : 0 }} />
                <span>{problems.warnings}</span>
              </>
            )}
          </div>
        )}
      </div>

      <div style={{ display: 'flex', gap: 15 }}>
        {/* Current File */}
        {currentFile && (
          <div className="status-bar-item">
            <span>{currentFile}</span>
          </div>
        )}

        {/* Cursor Position */}
        <div className="status-bar-item">
          <span>Ln {position.line}, Col {position.column}</span>
        </div>

        {/* Consensus Status */}
        <div className="status-bar-item">
          {getStatusIcon()}
          <span style={{ marginLeft: 5 }}>
            {consensusStatus === 'running' ? 'Running Consensus...' : 'Ready'}
          </span>
        </div>
      </div>
    </div>
  );
};