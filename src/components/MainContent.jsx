import React, { useState, useEffect, useCallback } from 'react';
import { Button, notification, Spin, Typography, List, Card, Tag, message } from 'antd';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { useGameConfig } from '../contexts/GameConfigContext';

const { Title, Text, Paragraph } = Typography;

// --- Setup Overlay Component ---
const SetupOverlay = ({ onSetupComplete }) => {
  const handleSetup = async () => {
    try {
      const selectedPath = await openDialog({
        multiple: false,
        title: 'Select MonsterHunterWilds.exe',
        filters: [{ name: 'Executable', extensions: ['exe'] }]
      });

      if (selectedPath && typeof selectedPath === 'string') {
        console.log('Selected executable:', selectedPath);
        await invoke('finalize_setup', { executablePath: selectedPath });
        notification.success({
          message: 'Setup Complete',
          description: 'Configuration saved successfully.',
          duration: 2
        });
        onSetupComplete();
      } else {
        console.log('No file selected or dialog cancelled.');
      }
    } catch (error) {
      console.error('Error during setup:', error);
      const errorMessage = typeof error === 'string' ? error : 'Failed to complete setup. Check console for details.';
      notification.error({ message: 'Setup Error', description: errorMessage });
    }
  };

  return (
    <div className="setup-overlay">
      <img src="/images/splashscreen.png" alt="Setup Background" className="setup-background-image" />
      <div className="setup-button-container">
        <Button type="text" onClick={handleSetup} className="setup-start-button">
          <span className="firstrun-add-path">Please select your game's executable.</span>
        </Button>
      </div>
      <style jsx>{`
        .setup-overlay {
          position: fixed; top: 0; left: 0; width: 100vw; height: 100vh;
          display: flex; justify-content: center; align-items: flex-end;
          z-index: 1000; cursor: default;
        }
        .setup-background-image {
          position: absolute; top: 0; left: 0; width: 100%; height: 100%;
          object-fit: cover; z-index: -1;
        }
        .setup-button-container { margin-bottom: 5vh; z-index: 1; }
        .setup-start-button {
          background-color: transparent !important; border: none !important;
          color: #90ee90 !important; padding: 10px 20px; font-size: 1.2em;
          font-family: 'CrimsonText-SemiBold', sans-serif; cursor: pointer;
          box-shadow: none !important; line-height: normal;
        }
        .setup-start-button:hover {
          color: #c1ffc1 !important; background-color: rgba(255, 255, 255, 0.1) !important;
        }
      `}</style>
    </div>
  );
};

// --- Main Content Component (Refactored) ---
const MainContent = () => {
  const { gameConfig, isLoading: isConfigLoading, error: configError, fetchGameConfig } = useGameConfig();

  const [installedMods, setInstalledMods] = useState([]);
  const [isModsLoading, setIsModsLoading] = useState(false);
  const [modsError, setModsError] = useState(null);
  const [isInstalling, setIsInstalling] = useState(false);

  const fetchMods = useCallback(async () => {
    if (!gameConfig) {
        console.log("fetchMods: No gameConfig yet.");
        return;
    }

    setIsModsLoading(true);
    setModsError(null);
    console.log("Attempting to invoke list_mods...");
    try {
      const mods = await invoke('list_mods', { gameRootPath: gameConfig.game_root_path });
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
  }, [gameConfig]);

  useEffect(() => {
    if (gameConfig) {
      console.log("useEffect: gameConfig available, calling fetchMods.");
      fetchMods();
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
      const selectedPaths = await openDialog({
        multiple: true,
        title: 'Select Mod Zip File(s)',
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
          fetchMods();
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
          fetchMods();
      } catch (err) {
          console.error(`Error toggling mod ${modName}:`, err);
          const errorMsg = typeof err === 'string' ? err : (err.message || 'Unknown error');
          message.error({ content: `Failed to toggle mod '${modName}': ${errorMsg}`, key: 'toggleMod', duration: 4 });
      }
  };
  // --- End New Handler ---

  if (isConfigLoading) {
    return (
      <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: 'calc(100vh - 150px)' }}>
        <Spin size="large" tip="Loading Configuration..." />
      </div>
    );
  }

  if (!configError && gameConfig === null) {
    return <SetupOverlay onSetupComplete={fetchGameConfig} />;
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
      <div className="main-content-container" style={{ padding: '24px' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' }}>
           <Title level={4} style={{ marginBottom: 0 }}>Installed Mods</Title>
           <div className="modinstall-button-container">
                <Button
                    onClick={handleInstallModFromZip}
                    loading={isInstalling}
                    type="primary"
                    style={{ marginRight: '8px' }}
                >
                    Install Mod from Zip
                </Button>
                <Button onClick={fetchMods} loading={isModsLoading}>
                    Refresh List
                </Button>
           </div>
        </div>

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
      </div>
    );
  }

  return (
    <div style={{ padding: '24px', textAlign: 'center' }}>
      <Text type="secondary">Waiting for configuration or encountering unexpected state...</Text>
    </div>
  );
};

export default MainContent; 