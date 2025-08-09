import React, { useState, useEffect } from 'react';
import { Modal, Tabs, Input, Button, Select, Form, Space, message, Typography, Alert, Spin, Card } from 'antd';
import { KeyOutlined, UserOutlined, SaveOutlined, CheckCircleOutlined } from '@ant-design/icons';
import { invoke } from '@tauri-apps/api/core';
import { useAppStore } from '../stores/appStore';
import { useConsensusStore } from '../stores/consensusStore';

const { TabPane } = Tabs;
const { Text, Title, Paragraph } = Typography;

interface SettingsDialogProps {
  visible: boolean;
  onClose: () => void;
}

export const SettingsDialog: React.FC<SettingsDialogProps> = ({ visible, onClose }) => {
  const [form] = Form.useForm();
  const [loading, setLoading] = useState(false);
  const [validating, setValidating] = useState(false);
  const { apiKeys, checkApiKeys } = useAppStore();
  const { profiles, activeProfile, loadProfiles, setActiveProfile } = useConsensusStore();
  
  // Local state for form
  const [openrouterKey, setOpenrouterKey] = useState('');
  const [hiveKey, setHiveKey] = useState('');
  const [selectedProfile, setSelectedProfile] = useState<string>('');

  useEffect(() => {
    if (visible) {
      // Load current settings when dialog opens
      loadCurrentSettings();
    }
  }, [visible]);

  const loadCurrentSettings = async () => {
    setLoading(true);
    try {
      // Load API keys status
      const keysStatus = await invoke<{
        openrouter: { configured: boolean };
        anthropic: { configured: boolean };
        hive: { configured: boolean };
      }>('get_api_keys_status');
      
      // Load profiles
      await loadProfiles();
      
      // Set active profile in form
      if (activeProfile) {
        setSelectedProfile(activeProfile.id);
      }
      
      // Update form with current values (keys are masked for security)
      form.setFieldsValue({
        openrouterKey: keysStatus.openrouter.configured ? '••••••••••••••••••••' : '',
        hiveKey: keysStatus.hive.configured ? '••••••••••••••••••••' : '',
        activeProfile: activeProfile?.id
      });
    } catch (error) {
      console.error('Failed to load settings:', error);
      message.error('Failed to load current settings');
    } finally {
      setLoading(false);
    }
  };

  const handleSaveApiKeys = async () => {
    setValidating(true);
    try {
      const values = form.getFieldsValue();
      
      // Only save if keys are not masked (user entered new values)
      if (values.openrouterKey && !values.openrouterKey.includes('•')) {
        // Validate OpenRouter key format
        if (!values.openrouterKey.startsWith('sk-or-v1-')) {
          throw new Error('Invalid OpenRouter key format. Keys should start with "sk-or-v1-"');
        }
        
        // Validate and save OpenRouter key
        const isValid = await invoke<boolean>('validate_api_key', {
          provider: 'openrouter',
          api_key: values.openrouterKey
        });
        
        if (!isValid) {
          throw new Error('Invalid OpenRouter API key. Please check your key and try again.');
        }
        
        await invoke('save_api_key_secure', {
          provider: 'openrouter',
          api_key: values.openrouterKey
        });
        
        message.success('OpenRouter API key saved successfully');
      }
      
      if (values.hiveKey && !values.hiveKey.includes('•')) {
        // Save Hive license key
        await invoke('save_api_key_secure', {
          provider: 'hive',
          api_key: values.hiveKey
        });
        
        message.success('Hive license key saved successfully');
      }
      
      // Refresh API keys status
      await checkApiKeys();
      
    } catch (error: any) {
      console.error('Failed to save API keys:', error);
      message.error(error.message || 'Failed to save API keys');
    } finally {
      setValidating(false);
    }
  };

  const handleProfileChange = async (profileId: string) => {
    try {
      await setActiveProfile(profileId);
      setSelectedProfile(profileId);
      message.success(`Profile changed to ${profiles.find(p => p.id === profileId)?.name}`);
    } catch (error) {
      console.error('Failed to change profile:', error);
      message.error('Failed to change profile');
    }
  };

  const handleSave = async () => {
    try {
      // Save API keys if changed
      await handleSaveApiKeys();
      
      // Save profile selection
      if (selectedProfile && selectedProfile !== activeProfile?.id) {
        await handleProfileChange(selectedProfile);
      }
      
      message.success('Settings saved successfully');
      onClose();
    } catch (error) {
      console.error('Failed to save settings:', error);
      message.error('Failed to save settings');
    }
  };

  return (
    <Modal
      title="Settings"
      visible={visible}
      onCancel={onClose}
      width={700}
      footer={[
        <Button key="cancel" onClick={onClose}>
          Cancel
        </Button>,
        <Button
          key="save"
          type="primary"
          icon={<SaveOutlined />}
          loading={validating}
          onClick={handleSave}
        >
          Save Settings
        </Button>
      ]}
    >
      <Spin spinning={loading}>
        <Tabs defaultActiveKey="api-keys">
          <TabPane tab="API Keys" key="api-keys">
            <Form
              form={form}
              layout="vertical"
              autoComplete="off"
            >
              <Alert
                message="API Key Security"
                description="Your API keys are encrypted and stored securely. Never share your API keys with anyone."
                type="info"
                showIcon
                style={{ marginBottom: 20 }}
              />
              
              <Form.Item
                label="OpenRouter API Key"
                name="openrouterKey"
                extra="Required for consensus engine. Get your key from openrouter.ai"
                rules={[
                  { required: !apiKeys.openrouter, message: 'OpenRouter API key is required' }
                ]}
              >
                <Input.Password
                  prefix={<KeyOutlined />}
                  placeholder={apiKeys.openrouter ? 'Key is configured (enter new key to update)' : 'sk-or-v1-...'}
                  onChange={(e) => setOpenrouterKey(e.target.value)}
                />
              </Form.Item>
              
              <Form.Item
                label="Hive License Key"
                name="hiveKey"
                extra="Optional. Unlocks premium features and cloud sync"
              >
                <Input.Password
                  prefix={<KeyOutlined />}
                  placeholder={apiKeys.hive ? 'Key is configured (enter new key to update)' : 'Enter your Hive license key'}
                  onChange={(e) => setHiveKey(e.target.value)}
                />
              </Form.Item>
              
              {apiKeys.openrouter && (
                <Alert
                  message="API Keys Configured"
                  description="Your OpenRouter API key is already configured. Enter a new key to update it."
                  type="success"
                  showIcon
                  icon={<CheckCircleOutlined />}
                />
              )}
            </Form>
          </TabPane>
          
          <TabPane tab="Consensus Profiles" key="profiles">
            <Card size="small" style={{ marginBottom: 16 }}>
              <Title level={5}>Active Profile</Title>
              <Paragraph>
                Select which consensus profile to use. Each profile optimizes the 4-stage consensus pipeline
                (Generator → Refiner → Validator → Curator) for different use cases.
              </Paragraph>
              
              <Form.Item
                label="Select Active Profile"
                name="activeProfile"
              >
                <Select
                  value={selectedProfile}
                  onChange={handleProfileChange}
                  style={{ width: '100%' }}
                  placeholder="Choose a consensus profile"
                >
                  <Select.OptGroup label="Speed Profiles">
                    {profiles.filter(p => p.id.includes('speed') || p.id.includes('fast')).map(profile => (
                      <Select.Option key={profile.id} value={profile.id}>
                        <Space>
                          <span>{profile.name}</span>
                          {profile.id === activeProfile?.id && <CheckCircleOutlined style={{ color: '#52c41a' }} />}
                        </Space>
                        <div style={{ fontSize: 12, color: '#888' }}>
                          G: {profile.generator_model.split('/').pop()} | 
                          R: {profile.refiner_model.split('/').pop()} | 
                          V: {profile.validator_model.split('/').pop()} | 
                          C: {profile.curator_model.split('/').pop()}
                        </div>
                      </Select.Option>
                    ))}
                  </Select.OptGroup>
                  
                  <Select.OptGroup label="Quality Profiles">
                    {profiles.filter(p => p.id.includes('quality') || p.id.includes('precision') || p.id.includes('architect')).map(profile => (
                      <Select.Option key={profile.id} value={profile.id}>
                        <Space>
                          <span>{profile.name}</span>
                          {profile.id === activeProfile?.id && <CheckCircleOutlined style={{ color: '#52c41a' }} />}
                        </Space>
                        <div style={{ fontSize: 12, color: '#888' }}>
                          G: {profile.generator_model.split('/').pop()} | 
                          R: {profile.refiner_model.split('/').pop()} | 
                          V: {profile.validator_model.split('/').pop()} | 
                          C: {profile.curator_model.split('/').pop()}
                        </div>
                      </Select.Option>
                    ))}
                  </Select.OptGroup>
                  
                  <Select.OptGroup label="Balanced Profiles">
                    {profiles.filter(p => p.id.includes('balanced') || p.id.includes('general')).map(profile => (
                      <Select.Option key={profile.id} value={profile.id}>
                        <Space>
                          <span>{profile.name}</span>
                          {profile.id === activeProfile?.id && <CheckCircleOutlined style={{ color: '#52c41a' }} />}
                        </Space>
                        <div style={{ fontSize: 12, color: '#888' }}>
                          G: {profile.generator_model.split('/').pop()} | 
                          R: {profile.refiner_model.split('/').pop()} | 
                          V: {profile.validator_model.split('/').pop()} | 
                          C: {profile.curator_model.split('/').pop()}
                        </div>
                      </Select.Option>
                    ))}
                  </Select.OptGroup>
                  
                  <Select.OptGroup label="Specialized Profiles">
                    {profiles.filter(p => 
                      !p.id.includes('speed') && !p.id.includes('fast') &&
                      !p.id.includes('quality') && !p.id.includes('precision') && !p.id.includes('architect') &&
                      !p.id.includes('balanced') && !p.id.includes('general') &&
                      !p.id.startsWith('custom_')
                    ).map(profile => (
                      <Select.Option key={profile.id} value={profile.id}>
                        <Space>
                          <span>{profile.name}</span>
                          {profile.id === activeProfile?.id && <CheckCircleOutlined style={{ color: '#52c41a' }} />}
                        </Space>
                        <div style={{ fontSize: 12, color: '#888' }}>
                          G: {profile.generator_model.split('/').pop()} | 
                          R: {profile.refiner_model.split('/').pop()} | 
                          V: {profile.validator_model.split('/').pop()} | 
                          C: {profile.curator_model.split('/').pop()}
                        </div>
                      </Select.Option>
                    ))}
                  </Select.OptGroup>
                  
                  {profiles.some(p => p.id.startsWith('custom_')) && (
                    <Select.OptGroup label="Custom Profiles">
                      {profiles.filter(p => p.id.startsWith('custom_')).map(profile => (
                        <Select.Option key={profile.id} value={profile.id}>
                          <Space>
                            <span>{profile.name}</span>
                            {profile.id === activeProfile?.id && <CheckCircleOutlined style={{ color: '#52c41a' }} />}
                          </Space>
                          <div style={{ fontSize: 12, color: '#888' }}>
                            G: {profile.generator_model.split('/').pop()} | 
                            R: {profile.refiner_model.split('/').pop()} | 
                            V: {profile.validator_model.split('/').pop()} | 
                            C: {profile.curator_model.split('/').pop()}
                          </div>
                        </Select.Option>
                      ))}
                    </Select.OptGroup>
                  )}
                </Select>
              </Form.Item>
              
              {activeProfile && (
                <Card size="small" style={{ marginTop: 16, background: '#f0f2f5' }}>
                  <Title level={5}>Current Profile: {activeProfile.name}</Title>
                  <Space direction="vertical" size="small" style={{ width: '100%' }}>
                    <Text><strong>Generator:</strong> {activeProfile.generator_model}</Text>
                    <Text><strong>Refiner:</strong> {activeProfile.refiner_model}</Text>
                    <Text><strong>Validator:</strong> {activeProfile.validator_model}</Text>
                    <Text><strong>Curator:</strong> {activeProfile.curator_model}</Text>
                  </Space>
                </Card>
              )}
            </Card>
          </TabPane>
        </Tabs>
      </Spin>
    </Modal>
  );
};