import React from 'react';
import {
  FolderOutlined,
  SearchOutlined,
  BranchesOutlined,
  BarChartOutlined,
  SettingOutlined,
  AppstoreOutlined,
} from '@ant-design/icons';
import { Tooltip } from 'antd';

interface ActivityBarProps {
  activeView: string;
  onViewChange: (view: string) => void;
}

export const ActivityBar: React.FC<ActivityBarProps> = ({ activeView, onViewChange }) => {
  const topItems = [
    { key: 'explorer', icon: <FolderOutlined />, tooltip: 'Explorer' },
    { key: 'search', icon: <SearchOutlined />, tooltip: 'Search' },
    { key: 'git', icon: <BranchesOutlined />, tooltip: 'Source Control' },
    { key: 'analytics', icon: <BarChartOutlined />, tooltip: 'Analytics' },
    { key: 'extensions', icon: <AppstoreOutlined />, tooltip: 'Extensions' },
  ];

  return (
    <div className="activity-bar">
      <div className="activity-bar-content">
        {topItems.map((item) => (
          <Tooltip key={item.key} title={item.tooltip} placement="right">
            <div
              className={`activity-bar-item ${activeView === item.key ? 'active' : ''}`}
              onClick={() => onViewChange(item.key)}
            >
              <span style={{ fontSize: 20 }}>{item.icon}</span>
            </div>
          </Tooltip>
        ))}
      </div>
      <div className="activity-bar-bottom">
        <Tooltip title="Settings" placement="right">
          <div
            className={`activity-bar-item ${activeView === 'settings' ? 'active' : ''}`}
            onClick={() => onViewChange('settings')}
          >
            <span style={{ fontSize: 20 }}><SettingOutlined /></span>
          </div>
        </Tooltip>
      </div>
    </div>
  );
};