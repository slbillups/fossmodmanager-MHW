import React, { useState } from 'react';
import { Input, Button, Form, Space, Typography, List, Checkbox, Row, Col, Switch, Modal, Divider, Tooltip } from 'antd';
import { DeleteOutlined, QuestionCircleOutlined, EditOutlined, SaveOutlined, CloseOutlined } from '@ant-design/icons'; // Added EditOutlined, SaveOutlined, CloseOutlined

const { Title, Text } = Typography;

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
  const [defaultPrefix, setDefaultPrefix] = useState('~/.local/share/Steam/SteamApps/compatdata/');
  const [defaultPrefixChanged, setDefaultPrefixChanged] = useState(false);
  const [autoUpdatePaths, setAutoUpdatePaths] = useState(true);
  const [currentGameData, setCurrentGameData] = useState(gameData);
  const [editingItemId, setEditingItemId] = useState(null); // ID of item being edited
  const [editWinePrefix, setEditWinePrefix] = useState(''); // Temp value for wine prefix input
  const [editProtonPath, setEditProtonPath] = useState(''); // Temp value for proton path input

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

  return (
    <div style={{ padding: '24px' }}>
      <Title level={3}>Settings</Title>

      {/* Default Wine Prefix Section */}
      <Form layout="vertical" style={{ marginBottom: '32px' }}>
        <Form.Item 
           label={
             <Space>
               <Title level={4} style={{marginBottom: 0}}>Default wine prefix</Title>
               <Tooltip title="The base directory where Wine/Proton creates virtual C: drives for games.">
                 <QuestionCircleOutlined style={{color: 'rgba(255, 255, 255, 0.45)'}}/>
               </Tooltip>
             </Space>
           }
           required
         >
          <Space.Compact style={{ width: '100%' }}>
            <Input
              value={defaultPrefix}
              onChange={handlePrefixChange}
              placeholder="Enter default Wine prefix path"
            />
            {defaultPrefixChanged && (
               <Button type="primary" onClick={handlePrefixSubmit}>Submit</Button>
            )}
          </Space.Compact>
           {!defaultPrefixChanged && (
               <Text type="secondary" style={{fontSize: '0.8em', marginLeft: '5px'}}>Path is current. Edit to enable submit.</Text>
           )}
        </Form.Item>
      </Form>

      <Divider /> {/* Divider 1 */}

      {/* Game Environment Variables Section */}
      <Title level={4} style={{ marginTop:'32px', marginBottom: '16px' }}>Your games env vars</Title>
      <Row gutter={16} style={{ marginBottom: '16px', paddingLeft: '8px', color: 'rgba(255, 255, 255, 0.45)' }}>
         <Col flex="auto"><Text strong>$WINEPREFIX</Text></Col>
         <Col flex="auto"><Text strong>$SPROTONPATH</Text></Col>
         <Col flex="100px" style={{textAlign: 'center'}}><Text strong>Actions</Text></Col> {/* Centered Actions */} 
      </Row>
      <List
        itemLayout="horizontal"
        dataSource={currentGameData}
        renderItem={(item) => {
          const isEditing = item.id === editingItemId;
          return (
            <List.Item
              actions={isEditing ? [
                  <Tooltip title="Save Changes">
                    <Button type="primary" shape="circle" icon={<SaveOutlined />} size="small" key={`save-${item.id}`} onClick={() => handleSaveItem(item.id)} />
                  </Tooltip>,
                  <Tooltip title="Cancel Edit">
                     <Button shape="circle" icon={<CloseOutlined />} size="small" key={`cancel-${item.id}`} onClick={handleCancelEdit} />
                  </Tooltip>
                ] : [
                  <Tooltip title="Remove Override">
                    <Button type="primary" danger shape="circle" icon={<DeleteOutlined />} size="small" key={`remove-${item.id}`} onClick={() => handleRemoveItem(item.id)} />
                  </Tooltip>,
                  <Tooltip title="Edit Paths">
                    <Button type="default" shape="circle" icon={<EditOutlined />} size="small" key={`update-${item.id}`} onClick={() => handleEditItem(item)} />
                  </Tooltip>,
              ]}
            >
              <List.Item.Meta
                title={item.name}
                description={
                  <Row gutter={16}>
                    <Col flex="auto">
                      {isEditing ? (
                        <Input 
                           size="small" 
                           value={editWinePrefix} 
                           onChange={(e) => handleEditInputChange(e, 'winePrefix')} 
                           placeholder="Enter Wine prefix path"
                        />
                      ) : (
                        <Input size="small" readOnly value={item.winePrefix} bordered={false} style={{backgroundColor: 'transparent', color: 'inherit'}}/>
                      )}
                    </Col>
                    <Col flex="auto">
                      {isEditing ? (
                        <Input 
                          size="small" 
                          value={editProtonPath} 
                          onChange={(e) => handleEditInputChange(e, 'protonPath')} 
                          placeholder="Enter Proton path"
                        />
                      ) : (
                        <Input size="small" readOnly value={item.protonPath} variant="filled" style={{backgroundColor: 'transparent', color: 'inherit'}}/>
                      )}
                    </Col>
                    <Col flex="100px"></Col> {/* Alignment spacer */}
                  </Row>
                }
              />
            </List.Item>
          );
        }}
        style={{ marginBottom: '32px' }}
      />

      <Divider /> {/* Divider 2 */}

      {/* Other Settings Section */} 
      <div style={{marginTop: '32px'}}>
         {/* Auto Update Section */}
         <Row justify="space-between" align="middle" style={{ marginBottom: '16px' }}>
           <Col>
             <Space>
                <Text>Automatically update your paths when started?</Text>
                <Tooltip title="If enabled, the manager will attempt to update game paths automatically on startup.">
                  <QuestionCircleOutlined style={{color: 'rgba(255, 255, 255, 0.45)'}}/>
                </Tooltip>
             </Space>
           </Col>
           <Col>
              <Switch
                checkedChildren="On"
                unCheckedChildren="Off"
                checked={autoUpdatePaths}
                onChange={handleAutoUpdateChange}
              />
            </Col>
         </Row>
  
         {/* Reset Section */}
         <Row justify="space-between" align="middle" style={{ marginBottom: '16px' }}>
           <Col>
             <Text>Reset? (Warning: destructive!)</Text>
           </Col>
           <Col>
             <Button type="primary" danger icon={<DeleteOutlined />} onClick={handleNuke}>
               Nuke it
             </Button>
           </Col>
         </Row>
       </div>

    </div>
  );
}

export default SettingsPage; 