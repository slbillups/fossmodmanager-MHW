import React, { useState, useEffect } from 'react';
import { Table, Tag, Checkbox, Button, Space, Tooltip, Avatar, notification } from 'antd';
import { open } from '@tauri-apps/plugin-dialog';
import { FolderOpenOutlined, QuestionCircleOutlined, ExclamationCircleOutlined, PlusOutlined } from '@ant-design/icons';
import { invoke } from '@tauri-apps/api/core';
import LocalMods from './LocalMods';
// Define columns based on original HTML table
const columns = [
  {
    title: 'Game',
    dataIndex: 'game_name',
    key: 'game_name',
    // Render the game name AND cover art
    render: (text, record) => (
      <Space>
        <Avatar
            shape="square"
            size={48} // Slightly larger avatar for cover art
            src={record.cover_art_data_url} // Use the field name from backend struct
            icon={<QuestionCircleOutlined />} // Fallback icon
        />
        <span>{text}</span>
      </Space>
    ),
  },
  {
    title: 'Version',
    dataIndex: 'version', // Now maps to buildid/version
    key: 'version',
  },
  {
    title: 'Game Root Path',
    dataIndex: 'game_root_path', // Use the field name from backend struct
    key: 'game_root_path',
    render: (path) => (
      <Tooltip title={path}> {/* Show full path in tooltip */}
        <span style={{ cursor: 'default' }}> {/* Use span instead of Button */}
          {/* Truncate paths to the last 3 directories */}
          {path.split('/').slice(-3).join('/')}
        </span>
      </Tooltip>
    ),
  },
];

// Remove mock game data (data)

// --- TODO: Add Mod Table Data & Expansion Logic --- 
// Mock mod data structure (keyed by game appid - needs adjustment)
const modData = {
  // Example using appid (assuming Dark Souls is 211420, DS2 is 236430)
  '211420': [
    { key: 'mod1', thumbUrl: '/placeholderimages/ds.jpg', name: 'DSFix', description: 'Graphics fixes and frame rate unlock.', needsUpdate: true },
    { key: 'mod2', thumbUrl: null, name: 'PVP Watchdog', description: 'Helps detect and manage online cheaters.', needsUpdate: false },
  ],
  '236430': [
    { key: 'mod3', thumbUrl: '/placeholderimages/ds2.jpg', name: 'GeDoSaTo', description: 'Advanced graphics tool (downsampling, etc.).', needsUpdate: false },
  ],
  // Add entries for other potential games by appid
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
];

const MainContent = () => {
  const [gameData, setGameData] = useState([]);
  const [expandedRowKeys, setExpandedRowKeys] = useState([]);
  const [loading, setLoading] = useState(false); // State for loading indicator

  // Function to fetch game list
  const fetchGameList = async () => {
    setLoading(true);
    try {
      // Call the renamed backend command
      const games = await invoke('load_game_list');
      // console.log('[DEBUG] Raw data from load_game_list:', JSON.stringify(games)); // Log raw data - REMOVE
      // console.log("Fetch completed, data received."); // Log completion - REMOVE

      // --- Restore data processing and state update ---
      // Prepare data for the table, using appid as the key
      const formattedData = games.map(game => ({
        ...game,
        key: game.appid, // Use appid from backend as the unique key
        // Add needsUpdate placeholder if needed later
        needsUpdate: false,
      }));

      // console.log('[DEBUG] Formatted data before setState:', JSON.stringify(formattedData)); // Log formatted data - REMOVE

      setGameData(formattedData);
      // --- End of restoring ---

    } catch (error) {
      console.error('Error loading game list:', error);
      notification.error({ message: 'Error Loading Games', description: String(error) });
      setGameData([]); // Clear data on error
    } finally {
      setLoading(false);
    }
  };

  // Effect to load game list on mount
  useEffect(() => {
    // fetchGameList(); // TEMPORARILY DISABLED
    fetchGameList(); // Re-enable the call
  }, []); // Empty dependency array ensures this runs only once on mount

  // Handler to add a new game
  const handleAddGame = async () => {
    try {
      // Open file dialog to select executable
      const selectedPath = await open({
        multiple: false,
        // Add filters for common executable types if desired
        // filters: [{ name: 'Executable', extensions: ['exe', 'bat', 'sh'] }]
        title: 'Select Game Executable',
      });

      if (selectedPath) {
        console.log('Selected executable:', selectedPath);
        setLoading(true);
        // Invoke the backend command to add the game
        const newGameData = await invoke('add_game', { executablePath: selectedPath });
        console.log('Added game data:', newGameData);

        // Add the new game to the state
        setGameData(prevData => [
            ...prevData,
            { ...newGameData, key: newGameData.appid, needsUpdate: false }
        ]);
        notification.success({ message: 'Game Added', description: `${newGameData.game_name} added successfully.` });

      } else {
        console.log('No file selected.');
      }
    } catch (error) {
      console.error('Error adding game:', error);
      // Display specific error from backend if available
      const errorMessage = typeof error === 'string' ? error : 'Failed to add game. Check console for details.';
      notification.error({ message: 'Error Adding Game', description: errorMessage });
    } finally {
      setLoading(false);
    }
  };

  // Determine if any game needs update (placeholder)
  // const isUpdateNeeded = gameData.some(item => item.needsUpdate);

  // Function to render the expanded row content (the mod table)
  const expandedRowRender = (gameRecord) => {
    // Use appid to fetch mods from modData
    const mods = modData[gameRecord.appid] || [];
    if (mods.length === 0) {
        // Optionally return a message instead of null
        return <div style={{ padding: '12px', color: '#888' }}>No mods configured for this game yet.</div>;
    }
    return <Table columns={modColumns} dataSource={mods} pagination={false} size="small" showHeader={false} />;
  };

  // Handler for row clicks to manage expansion
  const handleRowClick = (record) => {
    const currentKey = record.key; // key is now appid
    // Check modData using appid
    if (modData.hasOwnProperty(currentKey) && modData[currentKey].length > 0) {
      setExpandedRowKeys(prevKeys =>
        prevKeys.includes(currentKey)
          ? prevKeys.filter(k => k !== currentKey)
          : [...prevKeys, currentKey]
      );
    }
  };

  return (
    // Add a relative container to position the button against
    <div className="main-content-container" style={{ position: 'relative', minHeight: 'calc(100vh - 150px)' }}> {/* Adjust minHeight as needed */} 
      <Table
        columns={columns}
        dataSource={gameData}
        pagination={false}
        loading={loading} // Show loading indicator on table
        expandable={{
          expandedRowRender,
          // Use appid for rowExpandable check
          rowExpandable: (record) => modData.hasOwnProperty(record.appid) && modData[record.appid].length > 0,
          expandedRowKeys: expandedRowKeys,
        }}
        onRow={(record) => ({
          onClick: (event) => {
            // Prevent row click if button inside row was clicked
            if (event.target.closest('button')) return;
            handleRowClick(record);
          },
          style: {
            // Use appid for cursor style check
            cursor: (modData.hasOwnProperty(record.appid) && modData[record.appid].length > 0) ? 'pointer' : 'default'
          }
        })}
      />
      {/* Removed the bespoke update button for now */}
      {/* {isUpdateNeeded && ( ... ) } */}

      {/* Moved Add Game Button outside table, apply positioning class */}
      <div className="add-game-button-container">
         <Button
            type="primary"
            icon={<PlusOutlined />}
            onClick={handleAddGame}
            loading={loading}
         >
            Add Game
         </Button>
      </div>
    </div> // Close the relative container div
  );
};

export default MainContent; 