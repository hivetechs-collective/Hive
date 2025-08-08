import React, { useState, useEffect } from 'react';
import { Tree, Input, Dropdown, Menu, message } from 'antd';
import {
  FolderOutlined,
  FolderOpenOutlined,
  FileOutlined,
  FileTextOutlined,
  FileImageOutlined,
  CodeOutlined,
  SearchOutlined,
  PlusOutlined,
  DeleteOutlined,
  EditOutlined,
  ReloadOutlined,
} from '@ant-design/icons';
import { invoke } from '@tauri-apps/api/core';
import type { DataNode, EventDataNode } from 'antd/es/tree';

const { Search } = Input;

interface FileEntry {
  name: string;
  path: string;
  is_dir: boolean;
  size: number;
  modified?: number;
}

export const FileExplorer: React.FC = () => {
  const [treeData, setTreeData] = useState<DataNode[]>([]);
  const [expandedKeys, setExpandedKeys] = useState<string[]>([]);
  const [selectedKeys, setSelectedKeys] = useState<string[]>([]);
  const [searchValue, setSearchValue] = useState('');
  const [loading, setLoading] = useState(false);
  const [rootPath, setRootPath] = useState('/Users/veronelazio/Developer/Private/hive');

  useEffect(() => {
    loadDirectory(rootPath);
  }, [rootPath]);

  const getFileIcon = (name: string, isDir: boolean) => {
    if (isDir) {
      return expandedKeys.includes(name) ? <FolderOpenOutlined /> : <FolderOutlined />;
    }
    
    const ext = name.split('.').pop()?.toLowerCase();
    switch (ext) {
      case 'js':
      case 'jsx':
      case 'ts':
      case 'tsx':
      case 'rs':
      case 'py':
      case 'go':
        return <CodeOutlined style={{ color: '#519aba' }} />;
      case 'json':
      case 'toml':
      case 'yaml':
      case 'yml':
        return <FileTextOutlined style={{ color: '#cbcb41' }} />;
      case 'md':
      case 'txt':
        return <FileTextOutlined style={{ color: '#48a14d' }} />;
      case 'png':
      case 'jpg':
      case 'jpeg':
      case 'gif':
      case 'svg':
        return <FileImageOutlined style={{ color: '#c671e5' }} />;
      default:
        return <FileOutlined />;
    }
  };

  const loadDirectory = async (path: string) => {
    setLoading(true);
    try {
      const entries: FileEntry[] = await invoke('read_directory', { path });
      const nodes = entries.map(entry => ({
        title: entry.name,
        key: entry.path,
        icon: getFileIcon(entry.name, entry.is_dir),
        isLeaf: !entry.is_dir,
        children: entry.is_dir ? [] : undefined,
        data: entry,
      }));
      
      if (path === rootPath) {
        setTreeData(nodes);
      }
      
      return nodes;
    } catch (error) {
      message.error(`Failed to load directory: ${error}`);
      return [];
    } finally {
      setLoading(false);
    }
  };

  const onLoadData = async (node: EventDataNode<DataNode>) => {
    if (!node.children || node.children.length > 0) {
      return;
    }
    
    const children = await loadDirectory(node.key as string);
    setTreeData(origin => updateTreeData(origin, node.key as string, children));
  };

  const updateTreeData = (
    list: DataNode[],
    key: string,
    children: DataNode[]
  ): DataNode[] => {
    return list.map(node => {
      if (node.key === key) {
        return { ...node, children };
      }
      if (node.children) {
        return { ...node, children: updateTreeData(node.children, key, children) };
      }
      return node;
    });
  };

  const onSelect = async (keys: React.Key[], info: any) => {
    setSelectedKeys(keys as string[]);
    
    if (keys.length > 0 && !info.node.isLeaf) {
      return; // Don't open directories as files
    }
    
    if (keys.length > 0) {
      const filePath = keys[0] as string;
      // Emit event to open file in editor
      // This would be handled by the editor component
      console.log('Open file:', filePath);
    }
  };

  const onExpand = (keys: React.Key[]) => {
    setExpandedKeys(keys as string[]);
  };

  const onSearch = (value: string) => {
    setSearchValue(value);
    // Implement search filtering logic here
  };

  const contextMenu = (
    <Menu
      items={[
        {
          key: 'new-file',
          icon: <PlusOutlined />,
          label: 'New File',
          onClick: () => console.log('New file'),
        },
        {
          key: 'new-folder',
          icon: <FolderOutlined />,
          label: 'New Folder',
          onClick: () => console.log('New folder'),
        },
        { type: 'divider' },
        {
          key: 'rename',
          icon: <EditOutlined />,
          label: 'Rename',
          onClick: () => console.log('Rename'),
        },
        {
          key: 'delete',
          icon: <DeleteOutlined />,
          label: 'Delete',
          danger: true,
          onClick: () => console.log('Delete'),
        },
        { type: 'divider' },
        {
          key: 'refresh',
          icon: <ReloadOutlined />,
          label: 'Refresh',
          onClick: () => loadDirectory(rootPath),
        },
      ]}
    />
  );

  return (
    <div style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      <Search
        placeholder="Search files"
        onSearch={onSearch}
        onChange={(e) => onSearch(e.target.value)}
        style={{ marginBottom: 8, padding: '0 8px' }}
        prefix={<SearchOutlined />}
      />
      
      <div style={{ flex: 1, overflow: 'auto', padding: '0 8px' }}>
        <Dropdown menu={contextMenu as any} trigger={['contextMenu']}>
          <div style={{ height: '100%' }}>
            <Tree
              showIcon
              showLine={{ showLeafIcon: false }}
              loadData={onLoadData}
              treeData={treeData}
              expandedKeys={expandedKeys}
              selectedKeys={selectedKeys}
              onExpand={onExpand}
              onSelect={onSelect}
              style={{ background: 'transparent' }}
            />
          </div>
        </Dropdown>
      </div>
    </div>
  );
};