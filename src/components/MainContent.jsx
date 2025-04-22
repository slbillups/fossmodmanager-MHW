import React, { useState, useEffect, useCallback, useContext } from 'react';
import { Button, notification, Spin, Typography, List, Card, Tag, message, Layout } from 'antd';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { useGameConfig } from '../contexts/GameConfigContext';
import { GameConfigContext } from '../contexts/GameConfigContext';
import SetupOverlay from './SetupOverlay';
import InstallerCard from './InstallerCard';
import { DownloadOutlined } from '@ant-design/icons';
import CustomInstallButton from './CustomInstallButton';

const { Title, Text, Paragraph } = Typography;

// --- Main Content Component (Refactored) ---
const MainContent = () => {
  const { gameConfig, setGameConfig, isLoading: isConfigLoading, error: configError, fetchGameConfig } = useGameConfig();
  const { isLoading, setIsLoading, setError } = useContext(GameConfigContext);

  const [installedMods, setInstalledMods] = useState([]);
  const [isModsLoading, setIsModsLoading] = useState(false);
  const [modsError, setModsError] = useState(null);
  const [isInstalling, setIsInstalling] = useState(false);

  const fetchMods = useCallback(async (gameRootPath) => {
    if (!gameRootPath) return;
    try {
      setIsModsLoading(true);
      setModsError(null);
      console.log("Attempting to invoke list_mods...");
      const mods = await invoke('list_mods', { gameRootPath });
      console.log("Loaded mods:", mods);
      setInstalledMods(mods || []);
    } catch (err) {
      console.error('Error loading mods list:', err);
      const errorMsg = typeof err === 'string' ? err : (err.message || 'Unknown error');
      setModsError(`Failed to load mods list: ${errorMsg}`);
      notification.error({
        message: 'Mod List Error',
        description: `Failed to load mods: ${errorMsg}`,
        duration: 4
      });
      setInstalledMods([]);
    } finally {
      setIsModsLoading(false);
    }
  }, []);

  useEffect(() => {
    if (gameConfig) {
      console.log("useEffect: gameConfig available, calling fetchMods.");
      fetchMods(gameConfig.game_root_path);
    }
    if (!gameConfig) {
        console.log("useEffect: gameConfig is null, clearing installedMods.");
        setInstalledMods([]);
    }
  }, [gameConfig, fetchMods]);

  const getFilename = (fullPath) => {
    if (!fullPath) return 'unknown file';
    const lastSlash = fullPath.lastIndexOf('/');
    const lastBackslash = fullPath.lastIndexOf('\\');
    const lastSeparator = Math.max(lastSlash, lastBackslash);
    return lastSeparator === -1 ? fullPath : fullPath.substring(lastSeparator + 1);
  };

  const handleInstallModFromZip = async () => {
    if (!gameConfig?.game_root_path) {
      notification.error({
        message: 'Error',
        description: 'Game configuration not loaded. Cannot install mods.',
      });
      return;
    }

    try {
      const selectedPaths = await open({
        title: 'Select Mod Zip File(s)',
        multiple: true,
        directory: false,
        filters: [{ name: 'Zip Archives', extensions: ['zip'] }],
      });

      if (selectedPaths && selectedPaths.length > 0) {
        setIsInstalling(true);
        const installPromises = selectedPaths.map(async (zipPath) => {
          try {
            console.log(`Invoking install_mod_from_zip for: ${zipPath}`);
            await invoke('install_mod_from_zip', {
              zipPathStr: zipPath,
              gameRootPath: gameConfig.game_root_path,
            });
            message.success(`Successfully installed mod from ${getFilename(zipPath)}`);
            return { path: zipPath, success: true };
          } catch (error) {
            console.error(`Error installing mod from ${zipPath}:`, error);
            const errorMsg = typeof error === 'string' ? error : (error.message || 'Unknown error during installation');
            notification.error({
              message: 'Installation Error',
              description: `Failed to install mod from ${getFilename(zipPath)}: ${errorMsg}`,
              duration: 5
            });
            return { path: zipPath, success: false, error: errorMsg };
          }
        });

        const results = await Promise.allSettled(installPromises);
        console.log('Installation results:', results);

        const successfulInstalls = results.some(result => result.status === 'fulfilled' && result.value.success);
        if (successfulInstalls) {
          console.log("Install successful, calling fetchMods to refresh list.");
          fetchMods(gameConfig.game_root_path);
        }

      } else {
        console.log('No zip files selected.');
      }
    } catch (error) {
      console.error('Error opening file dialog:', error);
      notification.error({
        message: 'Dialog Error',
        description: 'Failed to open file selection dialog.',
      });
    } finally {
        setIsInstalling(false);
    }
  };

  // --- New Handler: Toggle Mod Enabled/Disabled ---
  const handleToggleMod = async (modName, currentStatus) => {
      if (!gameConfig?.game_root_path) {
          message.error('Game config not loaded.');
          return;
      }

      const enable = !currentStatus;
      const actionText = enable ? 'Enabling' : 'Disabling';
      message.loading({ content: `${actionText} mod '${modName}'...`, key: 'toggleMod' });

      try {
          await invoke('toggle_mod_enabled_state', {
              gameRootPath: gameConfig.game_root_path,
              modName: modName,
              enable: enable,
          });
          message.success({ content: `Mod '${modName}' ${enable ? 'enabled' : 'disabled'}.`, key: 'toggleMod', duration: 2 });
          // Refresh the list after successful toggle
          fetchMods(gameConfig.game_root_path);
      } catch (err) {
          console.error(`Error toggling mod ${modName}:`, err);
          const errorMsg = typeof err === 'string' ? err : (err.message || 'Unknown error');
          message.error({ content: `Failed to toggle mod '${modName}': ${errorMsg}`, key: 'toggleMod', duration: 4 });
      }
  };
  // --- End New Handler ---

  // New handler for completing the setup process
  const handleSetupComplete = async (validatedData) => {
    if (!validatedData) {
      console.error('handleSetupComplete called without validated data.');
      notification.error({ message: 'Setup Error', description: 'Internal error during setup completion.' });
      return;
    }
    try {
      console.log('Setup complete in parent, received data:', validatedData);
      // 1. Update the context state immediately
      setGameConfig(validatedData);

      // 2. Save the configuration persistently
      await invoke('save_game_config', { gameData: validatedData });
      console.log('Configuration saved successfully via save_game_config.');
      // Optionally show success feedback here if needed, though SetupOverlay already does

      // Optionally trigger initial fetch/refresh actions now that config is set and saved
      // fetchMods(validatedData.game_root_path);

    } catch (error) {
      console.error('Error saving game config:', error);
      const errorMsg = typeof error === 'string' ? error : 'Failed to save configuration.';
      notification.error({ message: 'Save Error', description: errorMsg });
      // Optionally clear the temporary config set in context if save fails
      // setGameConfig(null);
    }
  };

  if (isConfigLoading) {
    return (
      <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: 'calc(100vh - 150px)' }}>
        <Spin size="large" tip="Loading Configuration..." />
      </div>
    );
  }

  if (!gameConfig) {
    // Pass the new handler to SetupOverlay
    return <SetupOverlay onSetupComplete={handleSetupComplete} />;
  }

  if (configError) {
    return (
      <div style={{ padding: '24px', color: 'red', textAlign: 'center' }}>
        <Title level={4} style={{ color: 'red' }}>Configuration Error</Title>
        <Text type="danger">{configError}</Text>
        <br />
        <Text type="secondary">(Check context logs or restart. Ensure 'load_game_config' exists)</Text>
        <Button onClick={fetchGameConfig} style={{ marginTop: '16px' }}>Retry Load Config</Button>
      </div>
    );
  }

  if (!isConfigLoading && !configError && gameConfig) {
    return (
      <Layout style={{ minHeight: 'calc(100vh - 64px)' }}>
        <Layout.Content style={{ padding: '24px', background: '#000' }}>
          {/* truncate the game_root_path to the last 18 characters */}
          <Title level={5}>Game: {gameConfig.game_root_path.slice(-18)}</Title>

          <InstallerCard />



          <Title level={4}>Installed Mods</Title>

          {isModsLoading && !modsError && (
             <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', minHeight: '200px' }}>
                <Spin tip="Loading Mods..."></Spin>
             </div>
          )}

          {modsError && (
            <div style={{ color: 'orange', marginBottom: '10px', padding: '15px', border: '1px solid orange', borderRadius: '4px', textAlign: 'center' }}>
                <Text type="warning">Error loading mods: {modsError}</Text>
                <Button size="small" onClick={fetchMods} style={{ marginLeft: '8px' }}>Retry</Button>
            </div>
          )}

          {!isModsLoading && !modsError && (
            <List
                grid={{ gutter: 16, xs: 1, sm: 2, md: 3, lg: 4, xl: 5, xxl: 6 }}
                dataSource={installedMods}
                locale={{ emptyText: 'No mods installed yet. Use the "Install Mod from Zip" button to add some!' }}
                renderItem={(mod) => (
                    <List.Item>
                        <Card
                            title={mod.name || mod.directory_name}
                            size="small"
                        >
                             <Tag
                                 color={mod.enabled ? 'green' : 'red'}
                                 style={{ marginTop: '8px', cursor: 'pointer' }}
                                 onClick={() => handleToggleMod(mod.directory_name, mod.enabled)}
                             >
                                {mod.enabled ? 'Enabled' : 'Disabled'}
                             </Tag>
                        </Card>
                    </List.Item>
                )}
            />
          )}
                    <CustomInstallButton 
            onClick={handleInstallModFromZip}
            disabled={isInstalling}
            icon={<DownloadOutlined />}
            emphasized={installedMods.length === 0}
            style={{ marginBottom: 16, minWidth: '220px' }}
          >
            Install Mod(s) from Zip
          </CustomInstallButton>
        </Layout.Content>
      </Layout>
    );
  }

  return (
    <div style={{ padding: '24px', textAlign: 'center' }}>
      <Text type="secondary">Waiting for configuration or encountering unexpected state...</Text>
    </div>
  );
};

export default MainContent; 