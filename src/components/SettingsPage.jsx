import React, { useState, useEffect } from 'react';
import { Input, Button, Form, Space, Typography, List, Checkbox, Row, Col, Switch, Modal, Divider, Tooltip, Alert, Spin, Popconfirm, message, Card } from 'antd';
import { DeleteOutlined, QuestionCircleOutlined, EditOutlined, SaveOutlined, CloseOutlined, SyncOutlined, CheckCircleOutlined, SettingOutlined, ApiOutlined, ExclamationCircleFilled } from '@ant-design/icons';
import { invoke } from '@tauri-apps/api/core';
import { useGameConfig } from '../contexts/GameConfigContext';
import '../AppCustomStyles.css';
import ExtractGameAssets from './ExtractGameAssets';

const { Title, Text, Paragraph } = Typography;
const { confirm } = Modal;

function SettingsPage() {
  const { gameConfig, isLoading: isConfigLoading, error: configError, fetchGameConfig } = useGameConfig();
  const [checkForUpdatesOnLaunch, setCheckForUpdatesOnLaunch] = useState(false);
  const [apiKey, setApiKey] = useState('');
  const [isApiKeyValid, setIsApiKeyValid] = useState(null);
  const [isValidating, setIsValidating] = useState(false);
  const [checkUpdates, setCheckUpdates] = useState(true);
  const [enableAnalytics, setEnableAnalytics] = useState(false);

  useEffect(() => {
  }, []);

  const handleUpdateCheckChange = (checked) => {
    setCheckForUpdatesOnLaunch(checked);
    console.log(`Set check for updates on launch to: ${checked}`);
  };

  const handleValidateKey = async () => {
    if (!apiKey) {
      message.error('Please enter an API key.');
      return;
    }
    setIsValidating(true);
    setIsApiKeyValid(null);
    try {
      const validationResult = await invoke('validate_api_key', { apiKey });
      if (validationResult && validationResult.valid) {
        setIsApiKeyValid(true);
        message.success('API Key is valid!');
        // TODO: Save the validated key
      } else {
        setIsApiKeyValid(false);
        message.error(validationResult.reason || 'Invalid API Key.');
      }
    } catch (error) {
      console.error("API Key validation error:", error);
      message.error(`Failed to validate API key: ${error}`);
      setIsApiKeyValid(false);
    } finally {
      setIsValidating(false);
    }
  };

  const showDeleteConfirm = () => {
    confirm({
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
      await invoke('delete_config');
      message.success('Configuration deleted. Reloading configuration...');
      fetchGameConfig();
    } catch (error) {
      console.error("Failed to delete config:", error);
      message.error(`Error resetting settings: ${error}`);
    }
  };

  const handleOpenModsFolder = () => {
    if (!gameConfig || !gameConfig.game_root_path) {
      message.error("Game configuration not loaded, cannot open mods folder.");
      return;
    }
    console.log(`Requesting to open mods folder: ${gameConfig.game_root_path}/fossmodmanager/mods`);
    invoke('open_mods_folder', { gameRootPath: gameConfig.game_root_path })
      .then(() => console.log('Open mods folder command sent.'))
      .catch(err => {
        console.error("Failed to open mods folder:", err);
        message.error(`Error opening mods folder: ${err}`);
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

      <Card title="Installed Mods" style={{ marginBottom: 24 }}>
        <Paragraph type="secondary">
          No mods detected (or functionality not yet implemented).
        </Paragraph>
        <Button onClick={() => console.log('Refresh Mods clicked (not implemented yet)')}>
          Refresh Mods
        </Button>
      </Card>

      <Card title={<><ApiOutlined /> Nexus Mods API Key</>} style={{ marginBottom: 24 }}>
        <Paragraph type="secondary">
          Enter your personal Nexus Mods API key to enable features like checking endorsements and potentially downloading mods directly (if implemented).
          You can generate a key from your Nexus Mods profile page.
        </Paragraph>
        <Row gutter={8} align="middle">
          <Col flex="auto">
            <Input.Password
              placeholder="Enter your Nexus Mods API Key"
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
        {isApiKeyValid === true && <Text type="success" style={{ display: 'block', marginTop: 8 }}>Key is valid.</Text>}
        {isApiKeyValid === false && <Text type="danger" style={{ display: 'block', marginTop: 8 }}>Key is invalid.</Text>}
      </Card>

      <Card title="Application Settings" style={{ marginBottom: 24 }}>
        <Form layout="vertical">
          <Form.Item label="Check for Updates on Startup">
            <Checkbox checked={checkUpdates} onChange={(e) => setCheckUpdates(e.target.checked)}>
              Enable automatic update checks
            </Checkbox>
          </Form.Item>
          <Form.Item label="Analytics">
            <Checkbox checked={enableAnalytics} onChange={(e) => setEnableAnalytics(e.target.checked)}>
              Allow anonymous usage data collection (Optional - Helps improve the app)
            </Checkbox>
            <Paragraph type="secondary" style={{ fontSize: '0.85em' }}>
              We collect anonymous data like feature usage frequency and error reports to understand how the app is used and identify problems. No personal or game data is collected.
            </Paragraph>
          </Form.Item>
        </Form>
      </Card>

      <Card title={<><DeleteOutlined /> Reset Application</>} bordered={false} style={{ background: '#fffbe6', border: '1px solid #ffe58f' }}>
        <Paragraph type="warning">
          If you are experiencing persistent issues, you can reset the application's configuration.
          This action is irreversible and will require you to set up the game path again.
        </Paragraph>
        <Button type="primary" danger onClick={showDeleteConfirm}>
          Reset All Settings (Nuke)
        </Button>
      </Card>

    </div>
  );
}

export default SettingsPage; 