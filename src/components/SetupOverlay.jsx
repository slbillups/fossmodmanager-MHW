import React, { useState } from 'react';
import { Button, notification, Typography, Space, Spin } from 'antd';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import styles from './cssmodules/arkveldrune.module.css';
const { Title, Paragraph, Text } = Typography;

// THIS PAGE SHOULD ONLY APPEAR DURING THE FIRST LAUNCH OF THE APP

// if the user has already run the app before and the userconfig.json exists with a valid path to the game directory
//  this page should not appear
const SetupOverlay = ({ onSetupComplete }) => {
  const [selectedPathDisplay, setSelectedPathDisplay] = useState('No file selected...');
  const [isProcessing, setIsProcessing] = useState(false);
  const [error, setError] = useState(null);

  const handleSetup = async () => {
    setIsProcessing(true);
    setError(null);
    try {
      const selectedPath = await open({
        multiple: false,
        directory: false,
        title: 'Select Game Executable (e.g., MHWilds.exe)',
        // filters: [{ name: 'Executable', extensions: ['exe'] }] // Optional filters
      });

      if (selectedPath && typeof selectedPath === 'string') {
        console.log('Selected executable:', selectedPath);
        setSelectedPathDisplay(selectedPath);

        // Step 1: Validate the installation
        const validatedData = await invoke('validate_game_installation', { executablePath: selectedPath });
        console.log('Validation successful:', validatedData);

        // Step 2: Call the callback to save the config
        // The parent (GameConfigProvider) will handle setIsLoading(false) after saving/reloading
        await onSetupComplete(validatedData);

        notification.success({
          message: 'Game Path Validated',
          description: 'Game location confirmed. Saving configuration...',
          duration: 2
        });

      } else {
        console.log('No file selected or dialog cancelled.');
        setIsProcessing(false); // Stop processing if cancelled
      }
    } catch (error) {
      console.error('Error during setup validation:', error);
      const errorMessage = typeof error === 'string' ? error : `Failed to validate game path: ${error}`;
      setError(errorMessage);
      notification.error({ message: 'Setup Error', description: errorMessage });
      setIsProcessing(false); // Stop processing on error
    }
    // No need to setIsProcessing(false) on success, as the component will unmount
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', height: '100vh', padding: '20px', textAlign: 'center' }}>
      <Spin spinning={isProcessing} tip="Validating...">
        <Space direction="vertical" size="large">
          <Title level={2}>Initial Setup Required</Title>
          <Paragraph>
            Please select the main executable file for the game you want to manage.
            <br />
            (Example: <Text code>Game.exe</Text> or <Text code>game-binary</Text>)
          </Paragraph>

          <Space direction="vertical" align="center">
            <Button style={styles.arkveldButton} type="primary" onClick={handleSetup} disabled={isProcessing}>
              Browse for Game Executable...
            </Button>
            <Text type="secondary" style={{ marginTop: '10px', minHeight: '1.2em', maxWidth: '600px', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
              Selected: {selectedPathDisplay}
            </Text>
          </Space>

          {error && (
            <Typography.Text type="danger" style={{ marginTop: '15px' }}>
              Error: {error}
            </Typography.Text>
          )}
        </Space>
      </Spin>
    </div>
  );
};

export default SetupOverlay; 