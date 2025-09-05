import React, { useState, useEffect } from 'react';
import { 
  Modal, Tabs, Input, Button, Select, Form, Space, message, Typography, 
  Alert, Spin, Card, Switch, Slider, Radio, Divider, List, Tag, InputNumber 
} from 'antd';
import { 
  KeyOutlined, UserOutlined, SaveOutlined, CheckCircleOutlined,
  ThunderboltOutlined, DollarOutlined, ExperimentOutlined,
  SettingOutlined, CodeOutlined, GlobalOutlined, DatabaseOutlined,
  RocketOutlined, SafetyOutlined, BellOutlined
} from '@ant-design/icons';
import { invoke } from '@tauri-apps/api/core';
import { useAppStore } from '../stores/appStore';
import { useConsensusStore } from '../stores/consensusStore';

const { TabPane } = Tabs;
const { Text, Title, Paragraph } = Typography;
const { Option } = Select;

interface SettingsDialogProps {
  visible: boolean;
  onClose: () => void;
}

export const SettingsDialogEnhanced: React.FC<SettingsDialogProps> = ({ visible, onClose }) => {
  const [form] = Form.useForm();
  const [loading, setLoading] = useState(false);
  const { apiKeys, checkApiKeys, settings, updateSettings } = useAppStore();
  const { profiles, activeProfile, loadProfiles, setActiveProfile } = useConsensusStore();
  
  const [activeTab, setActiveTab] = useState('general');

  useEffect(() => {
    if (visible) {
      loadCurrentSettings();
    }
  }, [visible]);

  const loadCurrentSettings = async () => {
    setLoading(true);
    try {
      await checkApiKeys();
      await loadProfiles();
      
      // Load all settings
      const currentSettings = await invoke<any>('get_settings');
      form.setFieldsValue(currentSettings);
    } catch (error) {
      console.error('Failed to load settings:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleSave = async () => {
    try {
      const values = form.getFieldsValue();
      await invoke('save_settings', { config: values });
      updateSettings(values);
      message.success('Settings saved successfully');
      onClose();
    } catch (error) {
      message.error('Failed to save settings');
    }
  };

  return (
    <Modal
      title={
        <Space>
          <SettingOutlined />
          <span>Settings</span>
        </Space>
      }
      open={visible}
      onCancel={onClose}
      width={800}
      footer={[
        <Button key="cancel" onClick={onClose}>Cancel</Button>,
        <Button key="save" type="primary" icon={<SaveOutlined />} onClick={handleSave} loading={loading}>
          Save Settings
        </Button>
      ]}
    >
      <Tabs activeKey={activeTab} onChange={setActiveTab}>
        {/* General Settings */}
        <TabPane 
          tab={<Space><SettingOutlined />General</Space>} 
          key="general"
        >
          <Form form={form} layout="vertical">
            <Form.Item label="Theme" name="theme">
              <Radio.Group>
                <Radio.Button value="dark">Dark</Radio.Button>
                <Radio.Button value="light">Light</Radio.Button>
                <Radio.Button value="auto">Auto</Radio.Button>
              </Radio.Group>
            </Form.Item>

            <Form.Item label="Auto Save" name="autoSave" valuePropName="checked">
              <Switch />
            </Form.Item>

            <Form.Item label="Auto Accept AI Operations" name="autoAccept" valuePropName="checked">
              <Switch />
            </Form.Item>

            <Form.Item label="Show Welcome Screen" name="showWelcome" valuePropName="checked">
              <Switch />
            </Form.Item>

            <Form.Item label="Font Size" name="fontSize">
              <Slider min={10} max={20} marks={{ 10: '10', 14: '14', 16: '16', 20: '20' }} />
            </Form.Item>
          </Form>
        </TabPane>

        {/* API Keys */}
        <TabPane 
          tab={<Space><KeyOutlined />API Keys</Space>} 
          key="api"
        >
          <Form form={form} layout="vertical">
            <Alert 
              message="API Key Security" 
              description="Your API keys are encrypted and stored locally. Never share them."
              type="info" 
              showIcon 
              style={{ marginBottom: 16 }}
            />

            <Form.Item 
              label="OpenRouter API Key" 
              name="openrouterKey"
              extra="Get your key from openrouter.ai/keys"
            >
              <Input.Password 
                placeholder="sk-or-v1-..." 
                prefix={<KeyOutlined />}
              />
            </Form.Item>

            <Form.Item 
              label="Anthropic API Key (Optional)" 
              name="anthropicKey"
              extra="For direct Claude access"
            >
              <Input.Password 
                placeholder="sk-ant-..." 
                prefix={<KeyOutlined />}
              />
            </Form.Item>

            <Form.Item 
              label="Hive License Key" 
              name="hiveKey"
              extra="Premium features access"
            >
              <Input.Password 
                placeholder="hive-..." 
                prefix={<KeyOutlined />}
              />
            </Form.Item>

            <Space>
              <Button onClick={() => invoke('validate_api_key', { provider: 'openrouter', api_key: form.getFieldValue('openrouterKey') })}>
                Test OpenRouter
              </Button>
              <Button onClick={() => invoke('validate_api_key', { provider: 'anthropic', api_key: form.getFieldValue('anthropicKey') })}>
                Test Anthropic
              </Button>
            </Space>
          </Form>
        </TabPane>

        {/* Profiles & Models */}
        <TabPane 
          tab={<Space><UserOutlined />Profiles</Space>} 
          key="profiles"
        >
          <Title level={5}>Available Profiles</Title>
          <List
            dataSource={profiles}
            renderItem={(profile: any) => (
              <List.Item
                actions={[
                  <Button 
                    type={profile.id === activeProfile?.id ? 'primary' : 'default'}
                    onClick={() => setActiveProfile(profile)}
                  >
                    {profile.id === activeProfile?.id ? 'Active' : 'Select'}
                  </Button>
                ]}
              >
                <List.Item.Meta
                  title={
                    <Space>
                      {profile.name}
                      {profile.is_custom && <Tag color="orange">Custom</Tag>}
                      <Tag color={
                        profile.category === 'Speed' ? 'green' : 
                        profile.category === 'Quality' ? 'blue' : 
                        'purple'
                      }>
                        {profile.category}
                      </Tag>
                    </Space>
                  }
                  description={profile.description}
                />
              </List.Item>
            )}
          />
          
          <Divider />
          
          <Button type="dashed" block>
            + Create Custom Profile
          </Button>
        </TabPane>

        {/* Performance */}
        <TabPane 
          tab={<Space><ThunderboltOutlined />Performance</Space>} 
          key="performance"
        >
          <Form form={form} layout="vertical">
            <Form.Item label="Max Tokens per Request" name="maxTokens">
              <InputNumber min={100} max={8000} step={100} style={{ width: '100%' }} />
            </Form.Item>

            <Form.Item label="Temperature" name="temperature">
              <Slider min={0} max={1} step={0.1} marks={{ 0: '0', 0.5: '0.5', 1: '1' }} />
            </Form.Item>

            <Form.Item label="Response Streaming" name="streaming" valuePropName="checked">
              <Switch />
            </Form.Item>

            <Form.Item label="Cache Responses" name="cacheResponses" valuePropName="checked">
              <Switch />
            </Form.Item>

            <Form.Item label="Parallel Processing" name="parallel" valuePropName="checked">
              <Switch />
            </Form.Item>

            <Divider />

            <Title level={5}>Resource Limits</Title>
            
            <Form.Item label="Max Memory Usage (MB)" name="maxMemory">
              <InputNumber min={100} max={2000} step={100} style={{ width: '100%' }} />
            </Form.Item>

            <Form.Item label="Max CPU Cores" name="maxCores">
              <InputNumber min={1} max={16} step={1} style={{ width: '100%' }} />
            </Form.Item>
          </Form>
        </TabPane>

        {/* Cost Management */}
        <TabPane 
          tab={<Space><DollarOutlined />Cost</Space>} 
          key="cost"
        >
          <Form form={form} layout="vertical">
            <Alert 
              message="Cost Controls" 
              description="Set limits to prevent unexpected charges"
              type="warning" 
              showIcon 
              style={{ marginBottom: 16 }}
            />

            <Form.Item label="Daily Cost Limit ($)" name="dailyCostLimit">
              <InputNumber min={0} max={100} step={1} precision={2} style={{ width: '100%' }} />
            </Form.Item>

            <Form.Item label="Monthly Cost Limit ($)" name="monthlyCostLimit">
              <InputNumber min={0} max={1000} step={10} precision={2} style={{ width: '100%' }} />
            </Form.Item>

            <Form.Item label="Cost Warning Threshold (%)" name="costWarningThreshold">
              <Slider min={50} max={95} marks={{ 50: '50%', 75: '75%', 90: '90%', 95: '95%' }} />
            </Form.Item>

            <Form.Item label="Show Cost in Status Bar" name="showCostInStatusBar" valuePropName="checked">
              <Switch />
            </Form.Item>

            <Form.Item label="Require Confirmation Above ($)" name="confirmationThreshold">
              <InputNumber min={0} max={10} step={0.5} precision={2} style={{ width: '100%' }} />
            </Form.Item>
          </Form>
        </TabPane>

        {/* Advanced */}
        <TabPane 
          tab={<Space><ExperimentOutlined />Advanced</Space>} 
          key="advanced"
        >
          <Form form={form} layout="vertical">
            <Form.Item label="Enable Telemetry" name="telemetry" valuePropName="checked">
              <Switch />
            </Form.Item>

            <Form.Item label="Debug Mode" name="debug" valuePropName="checked">
              <Switch />
            </Form.Item>

            <Form.Item label="Experimental Features" name="experimental" valuePropName="checked">
              <Switch />
            </Form.Item>

            <Divider />

            <Title level={5}>Database</Title>

            <Form.Item label="Database Location" name="dbPath">
              <Input disabled placeholder="~/.hive/hive-ai.db" />
            </Form.Item>

            <Space>
              <Button onClick={() => invoke('vacuum_database')}>Vacuum Database</Button>
              <Button onClick={() => invoke('export_database')}>Export Data</Button>
              <Button danger onClick={() => invoke('reset_database')}>Reset Database</Button>
            </Space>

            <Divider />

            <Title level={5}>Network</Title>

            <Form.Item label="Proxy URL" name="proxyUrl">
              <Input placeholder="http://proxy.example.com:8080" />
            </Form.Item>

            <Form.Item label="Request Timeout (seconds)" name="requestTimeout">
              <InputNumber min={10} max={300} step={10} style={{ width: '100%' }} />
            </Form.Item>
          </Form>
        </TabPane>

        {/* Keyboard Shortcuts */}
        <TabPane 
          tab={<Space><CodeOutlined />Shortcuts</Space>} 
          key="shortcuts"
        >
          <Title level={5}>Keyboard Shortcuts</Title>
          <List
            dataSource={[
              { key: 'Cmd+K', action: 'Open Command Palette' },
              { key: 'Cmd+P', action: 'Quick Open File' },
              { key: 'Cmd+Shift+P', action: 'Show All Commands' },
              { key: 'Cmd+Enter', action: 'Run Consensus' },
              { key: 'Cmd+S', action: 'Save File' },
              { key: 'Cmd+,', action: 'Open Settings' },
              { key: 'Cmd+B', action: 'Toggle Sidebar' },
              { key: 'Cmd+J', action: 'Toggle Terminal' },
              { key: 'Cmd+\\', action: 'Split Editor' },
              { key: 'Cmd+W', action: 'Close Tab' },
            ]}
            renderItem={item => (
              <List.Item>
                <List.Item.Meta
                  title={<kbd>{item.key}</kbd>}
                  description={item.action}
                />
              </List.Item>
            )}
          />
        </TabPane>
      </Tabs>
    </Modal>
  );
};