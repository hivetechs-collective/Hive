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
  const items = [
    { key: 'explorer', icon: <FolderOutlined />, tooltip: 'Explorer' },
    { key: 'search', icon: <SearchOutlined />, tooltip: 'Search' },
    { key: 'git', icon: <BranchesOutlined />, tooltip: 'Source Control' },
    { key: 'analytics', icon: <BarChartOutlined />, tooltip: 'Analytics' },
    { key: 'extensions', icon: <AppstoreOutlined />, tooltip: 'Extensions' },
    { key: 'settings', icon: <SettingOutlined />, tooltip: 'Settings' },
  ];

  return (
    <div className="activity-bar">
      {items.map((item) => (
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
  );
};