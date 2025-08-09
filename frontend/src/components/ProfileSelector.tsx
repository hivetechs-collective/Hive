import React, { useState, useEffect } from 'react';
import { Select, Tooltip, Tag, Space, Spin } from 'antd';
import { UserOutlined, RocketOutlined, CheckCircleOutlined } from '@ant-design/icons';
import { invoke } from '@tauri-apps/api/core';
import { useConsensusStore } from '../stores/consensusStore';

const { Option } = Select;

interface ProfileInfo {
  id: string;
  name: string;
  description: string;
  category: string;
  is_active: boolean;
  is_custom: boolean;
  expert_level: string;
  use_cases: string[];
  tags: string[];
}

export const ProfileSelector: React.FC = () => {
  const [profiles, setProfiles] = useState<ProfileInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const { activeProfile, setActiveProfile } = useConsensusStore();

  useEffect(() => {
    loadProfiles();
  }, []);

  const loadProfiles = async () => {
    setLoading(true);
    try {
      const availableProfiles = await invoke<ProfileInfo[]>('get_available_profiles');
      setProfiles(availableProfiles);
      
      // Find and set the active profile
      const active = availableProfiles.find(p => p.is_active);
      if (active) {
        // Just set the profile name, the store will fetch the full profile
        await setActiveProfile(active.name);
      }
    } catch (error) {
      console.error('Failed to load profiles:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleProfileChange = async (profileId: string) => {
    setLoading(true);
    try {
      // Set the active profile in backend
      await invoke('set_active_profile', { profileId });
      
      // Get the updated profile config
      const profileData = await invoke<any>('get_profile_config', { profileId });
      const profile = profiles.find(p => p.id === profileId);
      
      if (profile) {
        // Just set the profile name, the store will fetch the full profile
        await setActiveProfile(profile.name);
      }
    } catch (error) {
      console.error('Failed to change profile:', error);
    } finally {
      setLoading(false);
    }
  };

  const getCategoryIcon = (category: string) => {
    switch (category) {
      case 'Speed':
        return <RocketOutlined style={{ color: '#52c41a' }} />;
      case 'Quality':
        return <CheckCircleOutlined style={{ color: '#1890ff' }} />;
      case 'Expert':
        return <UserOutlined style={{ color: '#722ed1' }} />;
      default:
        return <UserOutlined />;
    }
  };

  const getCategoryColor = (category: string) => {
    switch (category) {
      case 'Speed':
        return 'green';
      case 'Quality':
        return 'blue';
      case 'Expert':
        return 'purple';
      case 'Custom':
        return 'orange';
      default:
        return 'default';
    }
  };

  return (
    <div style={{ padding: '8px 16px', borderBottom: '1px solid var(--border-color)' }}>
      <Space>
        <span style={{ color: 'var(--text-secondary)', fontSize: 12 }}>Profile:</span>
        <Select
          value={activeProfile?.id}
          onChange={handleProfileChange}
          loading={loading}
          style={{ width: 200 }}
          placeholder="Select a profile"
          disabled={loading}
        >
          {profiles.map(profile => (
            <Option key={profile.id} value={profile.id}>
              <Tooltip title={profile.description} placement="left">
                <Space>
                  {getCategoryIcon(profile.category)}
                  <span>{profile.name}</span>
                  {profile.is_custom && <Tag color="orange">Custom</Tag>}
                  {profile.is_active && <Tag color="green">Active</Tag>}
                </Space>
              </Tooltip>
            </Option>
          ))}
        </Select>
        
        {activeProfile && (
          <Tag color={getCategoryColor(profiles.find(p => p.id === activeProfile.id)?.category || '')}>
            {profiles.find(p => p.id === activeProfile.id)?.category}
          </Tag>
        )}
        
        {loading && <Spin size="small" />}
      </Space>
    </div>
  );
};