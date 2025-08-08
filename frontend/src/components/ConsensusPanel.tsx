import React, { useState, useEffect, useRef } from 'react';
import { Card, Input, Button, Progress, Space, Typography, Spin, message } from 'antd';
import { SendOutlined, StopOutlined } from '@ant-design/icons';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { vscDarkPlus } from 'react-syntax-highlighter/dist/esm/styles/prism';
import { useConsensusStore } from '../stores/consensusStore';

const { TextArea } = Input;
const { Title, Text } = Typography;

interface ConsensusProgress {
  stage: string;
  progress: number;
  tokens: number;
  cost: number;
  message: string;
}

export const ConsensusPanel: React.FC = () => {
  const [query, setQuery] = useState('');
  const [loading, setLoading] = useState(false);
  const [currentStage, setCurrentStage] = useState<string>('');
  const [stageProgress, setStageProgress] = useState<Record<string, ConsensusProgress>>({});
  const [streamingContent, setStreamingContent] = useState<string>('');
  const [result, setResult] = useState<string>('');
  const resultRef = useRef<HTMLDivElement>(null);

  const stages = ['Generator', 'Refiner', 'Validator', 'Curator'];

  useEffect(() => {
    // Listen for consensus progress events
    const unlistenProgress = listen<ConsensusProgress>('consensus-progress', (event) => {
      const progress = event.payload;
      setCurrentStage(progress.stage);
      setStageProgress(prev => ({
        ...prev,
        [progress.stage]: progress
      }));
      
      // Append streaming content
      if (progress.message) {
        setStreamingContent(prev => prev + progress.message);
      }
    });

    // Listen for completion
    const unlistenComplete = listen<string>('consensus-complete', (event) => {
      setResult(event.payload);
      setLoading(false);
      message.success('Consensus completed successfully!');
    });

    // Listen for cancellation
    const unlistenCancel = listen<string>('consensus-cancelled', (event) => {
      setLoading(false);
      message.info(`Consensus cancelled: ${event.payload}`);
    });

    return () => {
      unlistenProgress.then(fn => fn());
      unlistenComplete.then(fn => fn());
      unlistenCancel.then(fn => fn());
    };
  }, []);

  const handleSubmit = async () => {
    if (!query.trim()) {
      message.warning('Please enter a query');
      return;
    }

    setLoading(true);
    setStreamingContent('');
    setResult('');
    setStageProgress({});

    try {
      await invoke('run_consensus_streaming', { query, window: window });
    } catch (error) {
      console.error('Consensus error:', error);
      message.error(`Failed to run consensus: ${error}`);
      setLoading(false);
    }
  };

  const handleCancel = async () => {
    try {
      await invoke('cancel_consensus');
    } catch (error) {
      console.error('Cancel error:', error);
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && e.ctrlKey) {
      handleSubmit();
    }
  };

  return (
    <Card 
      title="Consensus Engine" 
      style={{ margin: 16, height: 'calc(100% - 32px)' }}
      styles={{ body: { height: 'calc(100% - 60px)', display: 'flex', flexDirection: 'column' } }}
    >
      <Space direction="vertical" style={{ width: '100%', flex: 1 }} size="large">
        {/* Query Input */}
        <div>
          <TextArea
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyPress={handleKeyPress}
            placeholder="Enter your query here... (Ctrl+Enter to submit)"
            autoSize={{ minRows: 3, maxRows: 6 }}
            disabled={loading}
          />
          <Space style={{ marginTop: 8 }}>
            <Button
              type="primary"
              icon={<SendOutlined />}
              onClick={handleSubmit}
              loading={loading}
              disabled={!query.trim()}
            >
              Run Consensus
            </Button>
            {loading && (
              <Button
                danger
                icon={<StopOutlined />}
                onClick={handleCancel}
              >
                Cancel
              </Button>
            )}
          </Space>
        </div>

        {/* Stage Progress */}
        {loading && (
          <Card size="small" title="Progress">
            <Space direction="vertical" style={{ width: '100%' }}>
              {stages.map(stage => {
                const progress = stageProgress[stage];
                const isActive = currentStage === stage;
                const isComplete = progress?.progress === 100;
                
                return (
                  <div key={stage}>
                    <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 4 }}>
                      <Text strong={isActive}>{stage}</Text>
                      <Space>
                        {progress && (
                          <>
                            <Text type="secondary">{progress.tokens} tokens</Text>
                            <Text type="success">${progress.cost.toFixed(4)}</Text>
                          </>
                        )}
                      </Space>
                    </div>
                    <Progress
                      percent={progress?.progress || 0}
                      status={isActive ? 'active' : isComplete ? 'success' : 'normal'}
                      strokeColor={isActive ? '#1677ff' : undefined}
                    />
                  </div>
                );
              })}
            </Space>
          </Card>
        )}

        {/* Streaming Content */}
        {streamingContent && (
          <Card 
            size="small" 
            title="Streaming Output"
            styles={{ body: { maxHeight: 200, overflowY: 'auto' } }}
          >
            <pre style={{ whiteSpace: 'pre-wrap', wordBreak: 'break-word' }}>
              {streamingContent}
            </pre>
          </Card>
        )}

        {/* Final Result */}
        {result && (
          <Card 
            size="small" 
            title="Result"
            styles={{ body: { flex: 1, overflowY: 'auto' } }}
          >
            <div ref={resultRef}>
              <ReactMarkdown
                remarkPlugins={[remarkGfm]}
                components={{
                  code({ node, className, children, ...props }: any) {
                    const match = /language-(\w+)/.exec(className || '');
                    const isInline = !match;
                    return !isInline && match ? (
                      <SyntaxHighlighter
                        style={vscDarkPlus as any}
                        language={match[1]}
                        PreTag="div"
                        {...props}
                      >
                        {String(children).replace(/\n$/, '')}
                      </SyntaxHighlighter>
                    ) : (
                      <code className={className} {...props}>
                        {children}
                      </code>
                    );
                  }
                }}
              >
                {result}
              </ReactMarkdown>
            </div>
          </Card>
        )}
      </Space>
    </Card>
  );
};