import React, { useState } from 'react';
import { Table, Tag, Checkbox, Button, Space, Tooltip, Avatar } from 'antd';
import { FolderOpenOutlined, QuestionCircleOutlined, ExclamationCircleOutlined } from '@ant-design/icons';

// Define columns based on original HTML table
const columns = [
  {
    title: 'Game',
    dataIndex: 'game',
    key: 'game',
    // Render the game name (we'll add expand toggle later)
    render: (text) => <span>{text}</span>, 
  },
  {
    title: 'Version',
    dataIndex: 'version',
    key: 'version',
  },
  {
    title: 'Wine Prefix',
    dataIndex: 'prefixPath',
    key: 'prefixPath',
    render: (path) => (
      <Tooltip title={`Open prefix location: ${path}`}>
        <Button 
          type="link" 
          icon={<FolderOpenOutlined />} 
          onClick={() => console.log('Open prefix:', path)} // Placeholder action
          style={{ paddingLeft: 0 }} // Reduce padding for link button
        >
          {/* Maybe truncate long paths later */}
          {path} 
        </Button>
      </Tooltip>
    ),
  },
  {
    title: 'Needs Update?',
    dataIndex: 'needsUpdate',
    key: 'needsUpdate',
    render: (needsUpdate) => (
      needsUpdate 
        ? <ExclamationCircleOutlined style={{ color: 'orange' }} />
        : null
    ),
    align: 'center',
  },
];

// Mock data matching the columns
const data = [
  {
    key: '1',
    game: 'Dark Souls',
    version: '1.4.0a',
    prefixPath: '$HOME/dir/dir/dir1',
    needsUpdate: false,
  },
  {
    key: '2',
    game: 'Dark Souls II',
    version: '1.1.524',
    prefixPath: '$HOME/dir/dir/dir2',
    needsUpdate: true,
  },
  {
    key: '3',
    game: 'etc...',
    version: '1.1.524',
    prefixPath: '$HOME/dir/dir/dir3',
    needsUpdate: true,
  },
];

// --- TODO: Add Mod Table Data & Expansion Logic --- 
// Example structure for expanded data (mods for a game)
// const modData = {
//   '1': [ // Corresponds to game key '1' (Dark Souls)
//     { key: 'mod1', name: 'DSFix', description: 'Graphics fixes', enabled: true },
//     { key: 'mod2', name: 'PVP Watchdog', description: 'Anti-cheat', enabled: true },
//   ],
//   '2': [ // Corresponds to game key '2' (Dark Souls II)
//      { key: 'mod3', name: 'GeDoSaTo', description: 'Downsampling tool', enabled: false },
//   ]
// };

// Mock mod data structure (keyed by game key)
const modData = {
  '1': [
    { key: 'mod1', thumbUrl: '/placeholderimages/ds.jpg', name: 'DSFix', description: 'Graphics fixes and frame rate unlock.', needsUpdate: true },
    { key: 'mod2', thumbUrl: null, name: 'PVP Watchdog', description: 'Helps detect and manage online cheaters.', needsUpdate: false },
  ],
  '2': [
    { key: 'mod3', thumbUrl: '/placeholderimages/ds2.jpg', name: 'GeDoSaTo', description: 'Advanced graphics tool (downsampling, etc.).', needsUpdate: false },
  ],
  '3': [], // etc... has no mods in this example
};

// Columns for the inner mod table
const modColumns = [
  {
    dataIndex: 'name',
    key: 'name',
    render: (text, record) => (
      <Space>
        <Avatar shape="square" size="large" icon={<QuestionCircleOutlined />} src={record.thumbUrl} />
        <div>
          <div>{text}</div>
          <div style={{ color: '#888', fontSize: '0.9em' }}>{record.description}</div>
        </div>
      </Space>
    )
  },
  {
    title: 'Update? ', // Keep consistent with game table
    dataIndex: 'needsUpdate',
    key: 'needsUpdate',
    render: (needsUpdate) => (
      needsUpdate 
        ? <ExclamationCircleOutlined style={{ color: 'orange' }} /> 
        : null
    ),
    align: 'center',
    width: 100, // Give it a fixed width
  },
  // Add Actions column later (e.g., enable/disable, configure, delete)
];

const MainContent = () => {
  // State to keep track of the keys of expanded rows
  const [expandedRowKeys, setExpandedRowKeys] = useState([]);

  // Function to render the expanded row content (the mod table)
  const expandedRowRender = (gameRecord) => {
    const mods = modData[gameRecord.key] || []; // Get mods for this game, default to empty array
    if (mods.length === 0) {
        return null;
    }
    return <Table columns={modColumns} dataSource={mods} pagination={false} size="small" />;
  };

  // Handler for row clicks to manage expansion
  const handleRowClick = (record) => {
    const currentKey = record.key;
    if (modData.hasOwnProperty(currentKey) && modData[currentKey].length > 0) {
      setExpandedRowKeys(prevKeys =>
        prevKeys.includes(currentKey)
          ? prevKeys.filter(k => k !== currentKey)
          : [...prevKeys, currentKey]
      );
    }
  };

  return (
    <>
      {/* Removed the Action Buttons Space */}
      {/* <Space style={{ marginBottom: 16 }}>
        <Button type="primary">Check for updates</Button>
      </Space> */}

      <Table
        columns={columns}
        dataSource={data}
        pagination={false}
        expandable={{
          expandedRowRender,
          rowExpandable: (record) => modData.hasOwnProperty(record.key) && modData[record.key].length > 0,
          expandedRowKeys: expandedRowKeys,
        }}
        onRow={(record) => ({
          onClick: (event) => {
            handleRowClick(record);
          },
          style: {
            cursor: (modData.hasOwnProperty(record.key) && modData[record.key].length > 0) ? 'pointer' : 'default'
          }
        })}
      />
    </>
  );
};

export default MainContent; 