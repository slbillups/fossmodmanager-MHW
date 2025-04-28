import React, { useState, useEffect } from 'react';
import { Input, Button, Form, Space, Typography, List, Checkbox, Row, Col, Switch, Modal, Divider, Tooltip, Alert, Spin, Popconfirm, message, Card } from 'antd';
import { DeleteOutlined, QuestionCircleOutlined, EditOutlined, SaveOutlined, CloseOutlined, SyncOutlined, CheckCircleOutlined, SettingOutlined, ApiOutlined, ExclamationCircleFilled } from '@ant-design/icons';
import { invoke } from '@tauri-apps/api/core';
import { relaunch } from '@tauri-apps/plugin-process';
import { useGameConfig } from '../contexts/GameConfigContext';
import '../AppCustomStyles.css';
import ExtractGameAssets from './ExtractGameAssets';
import { App } from 'antd';

const { Title, Text, Paragraph } = Typography;
const { confirm } = Modal;

function SettingsPage() {
  const { gameConfig, isLoading: isConfigLoading, error: configError, fetchGameConfig } = useGameConfig();
  const  setCheckForUpdatesOnLaunch = useState(false);
  const [apiKey, setApiKey] = useState('');
  const [isApiKeyValid, setIsApiKeyValid] = useState(null);
  const [isValidating, setIsValidating] = useState(false);

  const { modal, message: messageApi } = App.useApp();
  console.log('Modal instance:', modal);
  console.log('Message instance:', messageApi);

  useEffect(() => {
  }, []);

  const handleUpdateCheckChange = (checked) => {
    setCheckForUpdatesOnLaunch(checked);
    console.log(`Set check for updates on launch to: ${checked}`);
  };

  const handleValidateKey = async () => {
    if (!apiKey) {
      messageApi.error('Please enter an API key.');
      return;
    }
    setIsValidating(true);
    setIsApiKeyValid(null);
    try {
      const validationResult = await invoke('validate_api_key', { apiKey });
      if (validationResult && validationResult.valid) {
        setIsApiKeyValid(true);
        messageApi.success('API Key is valid!');
        // TODO: Save the validated key
      } else {
        setIsApiKeyValid(false);
        messageApi.error(validationResult.reason || 'Invalid API Key.');
      }
    } catch (error) {
      console.error("Not implemented yet", error);
      messageApi.error(`Not implemented yet: ${error}`);
      setIsApiKeyValid(false);
    } finally {
      setIsValidating(false);
    }
  };

  const showDeleteConfirm = () => {
    console.log('showDeleteConfirm function called!');
    modal.confirm({
      title: 'Are you sure you want to reset all settings?',
      icon: <ExclamationCircleFilled />,
      content: 'This will delete your configuration file (userconfig.json) and require you to set up the game path again.',
      okText: 'Yes, Reset Everything',
      okType: 'danger',
      cancelText: 'No, Cancel',
      onOk() {
        console.log('OK - Deleting config');
        handleNukeSettings();
      },
      onCancel() {
        console.log('Cancel reset');
      },
    });
  };

  const handleNukeSettings = async () => {
    try {
      await invoke('nuke_settings_and_relaunch');
      messageApi.success('Configuration deleted. Application will restart shortly...');
    } catch (error) {
      console.error("Failed to delete config or trigger relaunch:", error);
      const errorMessage = typeof error === 'string' ? error : `Error resetting settings: ${error}`;
      messageApi.error(errorMessage);
    }
  };

  const handleOpenModsFolder = () => {
    if (!gameConfig || !gameConfig.game_root_path) {
      messageApi.error("Game configuration not loaded, cannot open mods folder.");
      return;
    }
    console.log(`Requesting to open mods folder: ${gameConfig.game_root_path}/fossmodmanager/mods`);
    invoke('open_mods_folder', { gameRootPath: gameConfig.game_root_path })
      .then(() => console.log('Open mods folder command sent.'))
      .catch(err => {
        console.error("Failed to open mods folder:", err);
        messageApi.error(`Error opening mods folder: ${err}`);
      });
  };

  if (isConfigLoading) {
    return (
      <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '80vh' }}>
        <Spin size="large" tip="Loading Configuration..." />
      </div>
    );
  }

  if (configError && !gameConfig) {
    return (
      <div style={{ maxWidth: '800px', margin: 'auto', padding: '24px' }}>
        <Alert message="Error Loading Game Configuration" description={configError} type="error" showIcon />
        <Button onClick={fetchGameConfig} style={{ marginTop: '16px' }}>Retry Load</Button>
      </div>
    );
  }

  return (
    <div 
      className="settings-container"
      style={{ 
        maxWidth: '800px', 
        margin: 'auto', 
        padding: '24px',
        msOverflowStyle: 'none', /* IE and Edge */
        scrollbarWidth: 'none',  /* Firefox */
        overflow: 'auto'
      }}>
      <style>{`
        /* Hide scrollbars for this specific component */
        .settings-container::-webkit-scrollbar {
          display: none;
        }
        
        /* Hide scrollbars for cards */
        .ant-card::-webkit-scrollbar,
        .ant-card-body::-webkit-scrollbar {
          display: none;
        }
        
        /* Ensure the component can still scroll */
        .settings-container {
          -ms-overflow-style: none;
          scrollbar-width: none;
          overflow: auto;
        }
      `}</style>
      <Title level={3}><SettingOutlined /> Settings</Title>

      {gameConfig && (
        <Card title="Current Game Configuration" style={{ marginBottom: 24 }}>
          <Paragraph>
            <Text strong>Game Root Path: </Text><Text code>{gameConfig.game_root_path}</Text>
          </Paragraph>
          <Paragraph>
            <Text strong>Executable Path: </Text><Text code>{gameConfig.game_executable_path}</Text>
          </Paragraph>
          <Paragraph>
            <Text strong>Mods Directory: </Text><Text code>{`${gameConfig.game_root_path}/fossmodmanager/mods`}</Text>
          </Paragraph>
          <Space>
            <Button onClick={handleOpenModsFolder}>
              Open Mods Folder
            </Button>
            <ExtractGameAssets gameRoot={gameConfig?.game_root_path} />
          </Space>
        </Card>
      )}
      {!gameConfig && !isConfigLoading && !configError && (
         <Alert message="Game Configuration Not Set" description="Initial setup may be required." type="info" showIcon style={{ marginBottom: 24 }} />
      )}

      {/* opacity 40% */}
      <Card title="Games - Coming Soon(?)" style={{ marginBottom: 24, opacity: 0.4 }}>
        <Paragraph type="secondary">
          No games detected (or functionality not yet implemented).
        </Paragraph>
        <Button onClick={() => console.log('Refresh game list clicked (not implemented yet)')}>
          Refresh game list
        </Button>
      </Card>
      {/* opacity 40% */}
      <Card title={<><ApiOutlined /> Nexus Mods API Key - for now just use a .env file if you've forked this repo</>} style={{ marginBottom: 24, opacity: 0.4 }}>
        <Paragraph type="secondary">
          May end up adding this, depends on interest, whether or not NexusMods allow us to use their API, etc.
        </Paragraph>
        <Row gutter={8} align="middle">
          <Col flex="auto">
            <Input.Password
              placeholder="This currently does nothing at all!"
              value={apiKey}
              onChange={(e) => {
                setApiKey(e.target.value);
                setIsApiKeyValid(null);
              }}
            />
          </Col>
          <Col>
            <Button
              type="primary"
              onClick={handleValidateKey}
              loading={isValidating}
              disabled={!apiKey}
            >
              Validate & Save
            </Button>
          </Col>
        </Row>
        {isApiKeyValid === true && <Text type="success" style={{ display: 'block', marginTop: 8 }}>I told you already, this does absolutely nothing!</Text>}
        {isApiKeyValid === false && <Text type="danger" style={{ display: 'block', marginTop: 8 }}>Nope, still nothing!</Text>}
      </Card>


      <Card title={<><DeleteOutlined /> Reset Application</>} bordered={false} style={{ color: 'rgb(248, 73, 181)', background: 'rgb(126, 0, 0)', border: '1px solid rgb(255, 0, 0)' }}>
        <Paragraph style={{ fontSmoothing: 'antialiased', color: 'rgb(255, 255, 255)' }} type="warning">
          If you are experiencing persistent issues, you can reset the application's configuration.
          This action is irreversible and will require you to set up the game path again.
        </Paragraph>
        <Button className="reset-button" style={{ background: '#6c0740', border: '1px solid rgb(255, 2, 99)' }} type="primary" danger onClick={showDeleteConfirm}>
          Reset All Settings (Nuke)
        </Button>
      </Card>

    </div>
  );
}

export default SettingsPage; 