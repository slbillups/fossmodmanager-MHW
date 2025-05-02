import React, { useState } from 'react';
import { Button, notification, Typography, Spin } from 'antd';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
const { Title, Paragraph, Text } = Typography;
import './startupoverlay.css';

// THIS PAGE SHOULD ONLY APPEAR DURING THE FIRST LAUNCH OF THE APP

// if the user has already run the app before and the userconfig.json exists with a valid path to the game directory
//  this page should not appear
const SetupOverlay = ({ onSetupComplete }) => {
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
      });

      if (selectedPath && typeof selectedPath === 'string') {
        const validatedData = await invoke('validate_game_installation', { executablePath: selectedPath });
        await onSetupComplete(validatedData);
        notification.success({
          message: 'Game Path Validated',
          description: 'Game location confirmed. Saving configuration...',
          duration: 2
        });
      } else {
        setIsProcessing(false);
      }
    } catch (error) {
      const errorMsg = typeof error === 'string' ? error : `Failed to validate game path: ${error}`;
      setError(errorMsg);
      notification.error({ message: 'Setup Error', description: errorMsg });
      setIsProcessing(false);
    }
  };

  return (
    <div
      style={{
        minHeight: '100vh',
        width: '100vw',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        background: 'none',
        margin: 0,
        padding: 0,
      }}
    >
      <Spin spinning={isProcessing} tip="Validating...">
        <div
          style={{
            background: 'rgba(0,0,0,0.85)',
            borderRadius: 12,
            padding: '36px 32px 28px 32px',
            maxWidth: 420,
            width: '100%',
            boxShadow: '0 2px 24px 0 rgba(0,0,0,0.25)',
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
          }}
        >
          <Title level={3} style={{ color: '#fff', marginBottom: 12, fontWeight: 600, textAlign: 'center', width: '100%' }}>
            Initial Setup
          </Title>
          <Paragraph style={{ color: '#ccc', marginBottom: 18, textAlign: 'center', fontSize: 15, width: '100%' }}>
            Select your game executable to continue.
          </Paragraph>
          <div style={{ color: '#888', fontSize: 12, marginBottom: 22, width: '100%', textAlign: 'center' }}>
            <Text code style={{ background: 'rgba(255,255,255,0.07)', color: '#aaa', fontSize: 12, display: 'block', marginBottom: 4 }}>
              i.e if installed via Steam to the $HOME directory: $HOME/.local/share/Steam/SteamApps/common/MonsterHunterWilds/MonsterHunterWilds.exe
            </Text>
            <Text code style={{ background: 'rgba(255,255,255,0.07)', color: '#aaa', fontSize: 12, display: 'block' }}>
              or via flatpak: $HOME/.var/app/com.valvesoftware.Steam/.../MonsterHunterWilds.exe
            </Text>
          </div>
          <Button
            type="primary"
            onClick={handleSetup}
            disabled={isProcessing}
            style={{
              background: 'transparent',
              borderColor: '#52c41a',
              color: '#52c41a',
              fontWeight: 500,
              marginBottom: 16,
              width: '100%',
              maxWidth: 260,
              height: 40,
            }}
          >
            Browse for Game Executable
          </Button>
          {error && (
            <div
              style={{
                color: '#ff4d4f',
                fontSize: 13,
                marginTop: 8,
                textAlign: 'center',
                width: '100%',
              }}
            >
              {error}
            </div>
          )}
        </div>
      </Spin>
    </div>
  );
};

export default SetupOverlay; 