import React, { useState, useEffect } from 'react';
import { Input, Button, Form, Space, Typography, List, Checkbox, Row, Col, Switch, Modal, Divider, Tooltip, Alert, Spin, Popconfirm } from 'antd';
import { DeleteOutlined, QuestionCircleOutlined, EditOutlined, SaveOutlined, CloseOutlined, SyncOutlined, CheckCircleOutlined } from '@ant-design/icons'; // Added EditOutlined, SaveOutlined, CloseOutlined, SyncOutlined, CheckCircleOutlined
import { invoke } from '@tauri-apps/api/core';
import '../AppCustomStyles.css'; // Reuse existing styles if applicable

const { Title, Text, Paragraph } = Typography;

// Dummy data for game paths - replace with actual data later
const gameData = [
  {
    id: '1',
    name: 'Dark Souls',
    winePrefix: '/path/to/wineprefix/',
    protonPath: '/path/to/proton/',
  },
  {
    id: '2',
    name: 'Dark Souls II',
    winePrefix: '/path/to/wineprefix/',
    protonPath: '/path/to/proton/',
  },
  {
    id: '3',
    name: 'Elden Ring',
    winePrefix: '/some/other/prefix/',
    protonPath: '/a/different/proton/',
  },
];

function SettingsPage() {
  const [games, setGames] = useState([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState(null);
  const [checkForUpdatesOnLaunch, setCheckForUpdatesOnLaunch] = useState(false); // Placeholder state
  const [defaultPrefix, setDefaultPrefix] = useState('~/.local/share/Steam/SteamApps/compatdata/');
  const [defaultPrefixChanged, setDefaultPrefixChanged] = useState(false);
  const [autoUpdatePaths, setAutoUpdatePaths] = useState(true);
  const [currentGameData, setCurrentGameData] = useState(gameData);
  const [editingItemId, setEditingItemId] = useState(null); // ID of item being edited
  const [editWinePrefix, setEditWinePrefix] = useState(''); // Temp value for wine prefix input
  const [editProtonPath, setEditProtonPath] = useState(''); // Temp value for proton path input

  useEffect(() => {
    setIsLoading(true);
    invoke('load_game_list')
      .then(setGames)
      .catch(err => {
        console.error("Failed to load game list:", err);
        setError("Failed to load game list. Please check logs or add a game if none exist.");
      })
      .finally(() => setIsLoading(false));

    // TODO: Load update check preference from backend/store
    // invoke('load_setting', { key: 'checkForUpdatesOnLaunch' })
    //   .then(setCheckForUpdatesOnLaunch)
    //   .catch(console.error);

  }, []);

  const handlePrefixChange = (e) => {
    setDefaultPrefix(e.target.value);
    setDefaultPrefixChanged(true); // Mark as changed when user types
  };

  const handlePrefixSubmit = () => {
    console.log('Submitting new default prefix:', defaultPrefix);
    // Add verification logic here (e.g., call Tauri backend)
    setDefaultPrefixChanged(false); // Reset changed state after submit
  };

  const handleAutoUpdateChange = (checked) => {
    setAutoUpdatePaths(checked);
    console.log('Auto update paths:', checked);
  };

  const handleNuke = () => {
    Modal.confirm({
      title: 'Are you sure you want to reset all settings?',
      icon: <DeleteOutlined style={{ color: 'red' }}/>,
      content: 'This action is destructive and cannot be undone. It will reset the default prefix and potentially other stored configurations.',
      okText: 'Yes, Nuke it',
      okType: 'danger',
      cancelText: 'No, Cancel',
      onOk() {
        console.warn('Nuke confirmed! Implement actual reset logic here.');
        // Reset state (example)
        setDefaultPrefix('~/.local/share/Steam/SteamApps/compatdata/');
        setDefaultPrefixChanged(false);
        setAutoUpdatePaths(true);
        setCurrentGameData(gameData); // Reset game data if modified
        // TODO: Add calls to backend/Tauri to clear persistent storage
      },
      onCancel() {
        console.log('Nuke cancelled');
      },
    });
  };

  const handleRemoveItem = (id) => {
    Modal.confirm({
       title: 'Remove this game\'s path settings?',
       content: 'Are you sure you want to remove the WINEPREFIX and SPROTONPATH overrides for this game?',
       okText: 'Yes, Remove',
       okType: 'danger',
       cancelText: 'Cancel',
       onOk() {
         console.log('Remove item confirmed:', id);
         setCurrentGameData(currentGameData.filter(item => item.id !== id));
         // TODO: Add calls to backend/Tauri to update persistent storage
       },
       onCancel() {
         console.log('Remove cancelled');
       },
    });
  };

  // Start editing an item
  const handleEditItem = (item) => {
    setEditingItemId(item.id);
    setEditWinePrefix(item.winePrefix);
    setEditProtonPath(item.protonPath);
  };

  // Cancel editing
  const handleCancelEdit = () => {
    setEditingItemId(null);
    setEditWinePrefix('');
    setEditProtonPath('');
  };

  // Save edited item
  const handleSaveItem = (id) => {
    console.log('Saving item:', id, 'WinePrefix:', editWinePrefix, 'ProtonPath:', editProtonPath);
    setCurrentGameData(
      currentGameData.map(item =>
        item.id === id
          ? { ...item, winePrefix: editWinePrefix, protonPath: editProtonPath }
          : item
      )
    );
    // TODO: Add calls to backend/Tauri to update persistent storage
    setEditingItemId(null); // Exit edit mode
  };

  // Update temp state when editing inputs
  const handleEditInputChange = (e, field) => {
    if (field === 'winePrefix') {
      setEditWinePrefix(e.target.value);
    } else if (field === 'protonPath') {
      setEditProtonPath(e.target.value);
    }
  };

  const handleCleanup = (appid) => {
    console.log(`Cleanup clicked for appid: ${appid}`);
    // TODO: Implement backend call for cleanup
    alert(`Cleanup action for AppID ${appid} not implemented yet.`);
  };

  const handleVerify = (appid) => {
    console.log(`Verify clicked for appid: ${appid}`);
    // TODO: Implement backend call for verify
     alert(`Verify action for AppID ${appid} not implemented yet.`);
  };

  // Placeholder function for opening mods folder
  const handleOpenModsFolder = (appid) => {
    // TODO: Implement backend call to find and open the mods folder for this game (AppID: ${appid})
    console.log(`Open Mods Folder clicked for appid: ${appid}`);
    // alert(`Open Mods Folder action for AppID ${appid} not implemented yet. Backend logic needed.`);
    invoke('ensure_and_open_mods_folder', { appid: appid })
      .then(() => {
        console.log(`Successfully requested to open mods folder for appid: ${appid}`);
      })
      .catch(err => {
        console.error(`Failed to open mods folder for appid ${appid}:`, err);
        // Optionally show an error to the user
        alert(`Error opening mods folder: ${err}`);
      });
  };

  const handleResetNuke = () => {
    console.log("Reset/Nuke confirmed");
    // TODO: Implement backend call for reset/nuke
    alert("Reset/Nuke functionality not yet implemented.");
    // Optionally: Refresh game list or navigate away after successful nuke
    // setGames([]);
  };

  const handleUpdateCheckChange = (checked) => {
    setCheckForUpdatesOnLaunch(checked);
    console.log(`Set check for updates on launch to: ${checked}`);
    // TODO: Implement backend call to save setting
    // invoke('save_setting', { key: 'checkForUpdatesOnLaunch', value: checked })
    //  .catch(console.error);
  };

  if (isLoading) {
    return (
      <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '80vh', position: 'relative' }}>
        <Spin size="large" tip="Loading Game List..." />
      </div>
    );
  }

  if (error) {
     return (
       <div style={{ padding: '24px' }}>
          <Alert message="Error" description={error} type="error" showIcon />
       </div>
      );
  }

  return (
    <div style={{ padding: '24px' }}>
      {/* Header Row for Managed Games and Mods sections */}
      <Row align="middle" justify="space-between" style={{ marginBottom: '16px' }}>
        <Col flex="auto"> 
          {/* Wrap Title and Divider in a div with specific width */}
          <div style={{ width: '250px' }}> {/* Set an explicit width */}
            <Title level={3} style={{ marginBottom: '8px', marginTop: 0 }}>Managed Games</Title>
            <Divider style={{ marginTop: 0, marginBottom: 0}} /> 
          </div>
        </Col>
        <Col style={{ minWidth: '200px', textAlign: 'center', paddingRight: '7rem' }}> 
           <Title level={4} style={{ marginBottom: '8px', marginTop: 0 }}>Mods</Title>
           <Divider style={{ marginTop: 0, marginBottom: 0 }} /> 
        </Col>
      </Row>

      {/* Game List - Keep marginBottom if needed, or remove if the Row provides enough space */}
      <List
        style={{ marginBottom: '24px' }} // Adjust or remove as needed
        itemLayout="horizontal"
        dataSource={games}
        locale={{ emptyText: 'No games have been added yet. Add a game from the main Games tab.' }}
        renderItem={(game) => (
          <List.Item
            actions={[
              <Button key={`cleanup-${game.appid}`} icon={<SyncOutlined />} onClick={() => handleCleanup(game.appid)}>
                Cleanup
              </Button>,
              <Button key={`verify-${game.appid}`} icon={<CheckCircleOutlined />} onClick={() => handleVerify(game.appid)}>
                Verify
              </Button>,
              <Button key={`open-mods-${game.appid}`} onClick={() => handleOpenModsFolder(game.appid)}>
                Mods 🗁
              </Button>,
            ]}
          >
            <List.Item.Meta
              title={<Text strong>{game.game_name}</Text>}
              description={<Text type="secondary" style={{ wordBreak: 'break-all' }}>Path: {game.game_root_path}</Text>}
            />
          </List.Item>
        )}
      />

      <Divider />

      <Title level={3} style={{ marginTop: '32px', marginBottom: '24px' }}>Application Settings</Title>

        <div style={{ marginBottom: '24px', padding: '16px', background: 'rgba(255, 255, 255, 0.04)', borderRadius: '8px' }}>
             <Space align="center" style={{ display: 'flex', justifyContent: 'space-between' }}>
                <Space>
                    <Text>Check for pnpm/cargo updates on launch</Text>
                    <Tooltip title="Enable to automatically check for required dependency updates when the application starts.">
                        <QuestionCircleOutlined style={{ color: 'rgba(255, 255, 255, 0.45)' }} />
                    </Tooltip>
                </Space>
                 <Switch
                    checked={checkForUpdatesOnLaunch}
                    onChange={handleUpdateCheckChange}
                    checkedChildren="On"
                    unCheckedChildren="Off"
                 />
            </Space>
         </div>

        <div style={{ padding: '16px', background: 'rgba(255, 0, 0, 0.1)', borderRadius: '8px' }}>
            <Title level={4}>Reset Application</Title>
             <Paragraph type="secondary">
                This action is irreversible. It will remove all tracked games and revert all settings to their defaults.
             </Paragraph>
             <Popconfirm
                 title="Confirm Reset"
                 description="Are you absolutely sure you want to remove all data?"
                 onConfirm={handleResetNuke}
                 okText="Yes, Nuke It"
                 cancelText="Cancel"
                 okButtonProps={{ danger: true }}
             >
                 <Button type="primary" danger icon={<DeleteOutlined />}>
                     Reset / Nuke All Settings
                 </Button>
             </Popconfirm>
         </div>
    </div>
  );
}

export default SettingsPage; 