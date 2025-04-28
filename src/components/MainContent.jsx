import React, { useState, useEffect, useCallback, useContext, lazy, Suspense } from 'react';
import { Button, notification, Spin, Typography, List, Card, message, Layout } from 'antd';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke, Channel } from '@tauri-apps/api/core';
import { useGameConfig } from '../contexts/GameConfigContext';
import { GameConfigContext } from '../contexts/GameConfigContext';
import SetupOverlay from './SetupOverlay';
import InstallerCard from './InstallerCard';
import { 
  DownloadOutlined, 
  AppstoreOutlined, 
  SkinOutlined, 
  SettingOutlined,
  SearchOutlined
} from '@ant-design/icons';
import CustomInstallButton from './CustomInstallButton';

// Lazy load components to improve initial load time
const InstalledSkinMods = lazy(() => import('./SkinMods'));
const SettingsPage = lazy(() => import('./SettingsPage'));
const SearchPage = lazy(() => import('./SearchPage'));

const { Text } = Typography;

// --- Main Content Component (Refactored) ---
const MainContent = () => {
  const { gameConfig, setGameConfig, isLoading: isConfigLoading, error: configError, fetchGameConfig } = useGameConfig();
  const { isLoading, setIsLoading, setError } = useContext(GameConfigContext);

  const [installedMods, setInstalledMods] = useState([]);
  const [isModsLoading, setIsModsLoading] = useState( false);
  const [modsError, setModsError] = useState(null);
  const [isInstalling, setIsInstalling] = useState(false);
  const [currentTab, setCurrentTab] = useState('reframework');
  const [slideDirection, setSlideDirection] = useState(''); // 'left' or 'right'
  const [animating, setAnimating] = useState(false);
  const [pageCache, setPageCache] = useState({});
  const [assetsPreloaded, setAssetsPreloaded] = useState(false);

  const fetchMods = useCallback(async (gameRootPath) => {
    if (!gameRootPath) return;
    try {
      setIsModsLoading(true);
      setModsError(null);
      console.log("Attempting to invoke list_mods...");
      const mods = await invoke('list_mods', { gameRootPath });
      console.log("Loaded mods:", mods);
      setInstalledMods(mods || []);
      
      // Pre-load assets if not already done
      if (!assetsPreloaded) {
        preloadModAssets(mods);
      }
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
  }, [assetsPreloaded]);

  // Function to preload mod assets using Tauri's cache
  const preloadModAssets = async (mods) => {
    try {
      if (!mods || mods.length === 0) return;
      
      console.log("Preloading assets for mods...");
      await invoke('preload_mod_assets', { 
        mods: mods.map(mod => mod.directory_name) 
      });
      
      setAssetsPreloaded(true);
      console.log("Assets preloaded successfully");
    } catch (err) {
      console.error("Error preloading mod assets:", err);
      // Non-critical error, don't block the UI
    }
  };

  useEffect(() => {
    if (gameConfig) {
      console.log("useEffect: gameConfig available, calling fetchMods.");
      fetchMods(gameConfig.game_root_path);
    }
    if (!gameConfig) {
        console.log("useEffect: gameConfig is null, clearing installedMods.");
        setInstalledMods([]);
    }
  }, [gameConfig]);

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
        
        // Create a single channel to handle all events
        const channel = new Channel();
        
        // Simple event handler that just logs events
        channel.onmessage = (event) => {
          console.log('Installation event:', event);
          // You could update a global status message here if desired
        };
        
        const installPromises = selectedPaths.map(async (zipPath) => {
          try {
            await invoke('install_mod_from_zip', {
              zipPathStr: zipPath,
              gameRootPath: gameConfig.game_root_path,
              onEvent: channel
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
          fetchMods(gameConfig.game_root_path);
        }
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

    } catch (error) {
      console.error('Error saving game config:', error);
      const errorMsg = typeof error === 'string' ? error : 'Failed to save configuration.';
      notification.error({ message: 'Save Error', description: errorMsg });
    }
  };

  // Updated tab switching with lazy loading
  const handleTabChange = (newTab) => {
    if (newTab === currentTab) return;
    
    // Set direction based on tab order (reframework → skin = right, skin → reframework = left)
    const direction = 
      (currentTab === 'reframework' && newTab === 'skin') ? 'left' : 
      (currentTab === 'skin' && newTab === 'reframework') ? 'right' : '';
    
    setSlideDirection(direction);
    setAnimating(true);
    
    // Pre-cache the page content before showing it
    if (!pageCache[newTab]) {
      // This will trigger the lazy loading
      console.log(`Pre-caching ${newTab} tab content...`);
    }
    
    // Wait for animation before changing tab
    setTimeout(() => {
      setCurrentTab(newTab);
      // Reset after tab change
      setTimeout(() => {
        setAnimating(false);
      }, 50);
    }, 300); // Match this with CSS transition duration
  };

  const renderTabContent = () => {
    // Common loading spinner for lazy-loaded components
    const loadingFallback = (
      <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '200px' }}>
        <Spin size="large" tip="Loading..." />
      </div>
    );

    // REFramework Mods Tab
    if (currentTab === 'reframework' || (animating && (currentTab === 'skin' || slideDirection === 'right'))) {
      return (
        <div className={`tab-content ${animating && slideDirection === 'left' ? 'slide-left' : animating && slideDirection === 'right' ? 'tab-enter slide-right' : ''}`}>
          {/* Add mod item animation styles */}
          <style>
            {`
              .mod-item {
                border: 1px solid transparent;
                border-radius: 4px;
                position: relative;
                background: transparent;
                transition: background 0.3s ease;
              }
              
              .mod-item:hover {
                background: transparent;
              }
              
              .mod-item::before {
                content: "";
                position: absolute;
                top: 0;
                left: 0;
                right: 0;
                bottom: 0;
                border-radius: 4px;
                padding: 1px;
                background: linear-gradient(90deg, 
                  transparent 0%, 
                  transparent 50%, 
                  #52c41a 50%, 
                  #52c41a 60%, 
                  transparent 60%, 
                  transparent 100%
                );
                background-size: 200% 100%;
                background-position: 100% 0;
                transition: opacity 0.1s ease;
                opacity: 0;
                pointer-events: none;
                z-index: 1;
                mask: linear-gradient(#fff 0 0) content-box, linear-gradient(#fff 0 0);
                mask-composite: exclude;
                -webkit-mask: linear-gradient(#fff 0 0) content-box, linear-gradient(#fff 0 0);
                -webkit-mask-composite: xor;
              }
              
              .mod-item:hover::before {
                opacity: 1;
                animation: modBorderTrail 2s linear infinite;
                animation-delay: 0s;
              }
              
              @keyframes modBorderTrail {
                0% {
                  background-position: 200% 0;
                }
                100% {
                  background-position: 0% 0;
                }
              }
              
              .mod-status-indicator {
                transition: background-color 0.3s ease;
              }
            `}
          </style>
          
          {isModsLoading && !modsError && (
            <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', minHeight: '200px' }}>
              <Spin tip="Loading Mods..."></Spin>
            </div>
          )}

          {modsError && (
            <div style={{ 
              color: 'orange', 
              marginBottom: '16px', 
              padding: '15px', 
              background: 'rgba(255, 165, 0, 0.05)',
              border: '1px solid rgba(255, 165, 0, 0.2)', 
              borderRadius: '4px', 
              textAlign: 'center' 
            }}>
              <Text type="warning">Error loading mods: {modsError}</Text>
              <Button size="small" onClick={() => fetchMods(gameConfig?.game_root_path)} style={{ marginLeft: '8px' }}>Retry</Button>
            </div>
          )}

          {!isModsLoading && !modsError && (
            <div style={{ padding: '8px' }}>
              {installedMods.length === 0 ? (
                <div style={{ color: '#888', textAlign: 'center', marginTop: '24px' }}>
                  No mods installed yet. Use the "Install Mod from Zip" button to add some!
                </div>
              ) : (
                <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(220px, 1fr))', gap: '16px' }}>
                  {installedMods.map((mod) => (
                    <div 
                      key={mod.directory_name} 
                      onClick={() => handleToggleMod(mod.directory_name, mod.enabled)}
                      className="mod-item"
                      style={{ 
                        cursor: 'pointer',
                        position: 'relative',
                        padding: '12px',
                        marginBottom: '6px',
                        borderRadius: '4px'
                      }}
                    >
                      <div style={{ display: 'flex', alignItems: 'center' }}>
                        <div 
                          className="mod-status-indicator" 
                          style={{ 
                            width: '10px', 
                            height: '10px', 
                            background: mod.enabled ? '#52c41a' : '#444',
                            marginRight: '12px',
                            borderRadius: '2px',
                          }} 
                        />
                        <div style={{ flex: 1 }}>
                          <div style={{ 
                            fontSize: '15px', 
                            color: '#fff', 
                            fontWeight: 400,
                            marginBottom: '2px',
                            whiteSpace: 'nowrap',
                            overflow: 'hidden',
                            textOverflow: 'ellipsis'
                          }}>
                            {mod.name || mod.directory_name}
                          </div>
                          <div style={{ 
                            fontSize: '13px', 
                            color: mod.enabled ? '#52c41a' : '#777',
                            fontWeight: 300,
                            letterSpacing: '0.03em'
                          }}>
                            {mod.enabled ? 'Enabled' : 'Disabled'}
                          </div>
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}
        </div>
      );
    } 
    
    // Skin Mods Tab
    else if (currentTab === 'skin' || (animating && (currentTab === 'reframework' || slideDirection === 'left'))) {
      return (
        <div className={`tab-content ${animating && slideDirection === 'right' ? 'slide-right' : animating && slideDirection === 'left' ? 'tab-enter slide-left' : ''}`}>
          <Suspense fallback={loadingFallback}>
            <InstalledSkinMods gameRoot={gameConfig.game_root_path} />
          </Suspense>
        </div>
      );
    }
    
    // Search Tab - Fully lazy loaded
    else if (currentTab === 'search') {
      return (
        <Suspense fallback={loadingFallback}>
          <div style={{ 
            display: 'flex', 
            justifyContent: 'space-between', 
            alignItems: 'center',
            marginBottom: '16px'
          }}>
            <Text style={{ 
              color: '#ddd', 
              fontWeight: 500, 
              fontSize: '16px',
              letterSpacing: '0.5px'
            }}>
              Search Mods
            </Text>
          </div>
          
          <SearchPage />
        </Suspense>
      );
    }
    
    // Settings Tab - Fully lazy loaded
    else if (currentTab === 'settings') {
      return (
        <Suspense fallback={loadingFallback}>
          <SettingsPage />
        </Suspense>
      );
    }
    
    return null;
  };

  if (isConfigLoading) {
    return (
      <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: 'calc(100vh - 150px)' }}>
        <Spin size="large" tip="Loading Configuration..." />
      </div>
    );
  }

  if (configError) {
    return (
      <div style={{ padding: '24px', color: 'red', textAlign: 'center' }}>
        <Text type="danger">{configError}</Text>
        <br />
        <Text type="secondary">(Check context logs or restart. Ensure 'load_game_config' exists)</Text>
        <Button onClick={fetchGameConfig} style={{ marginTop: '16px' }}>Retry Load Config</Button>
      </div>
    );
  }

  if (!isConfigLoading && !configError && gameConfig) {
    return (
      <Layout style={{ 
        height: '100vh', 
        display: 'flex', 
        flexDirection: 'row', 
        background: '#000', 
        overflow: 'auto', 
        msOverflowStyle: 'none',
        scrollbarWidth: 'none'
      }}>
        {/* Add global style to hide scrollbars */}
        <style>
          {`
            /* Hide scrollbars but maintain scroll functionality */
            body, #root, .ant-layout {
              -ms-overflow-style: none;  /* IE and Edge */
              scrollbar-width: none;  /* Firefox */
              overflow: auto !important; /* Allow scrolling */
            }
            
            /* Hide WebKit scrollbars */
            body::-webkit-scrollbar,
            #root::-webkit-scrollbar,
            .ant-layout::-webkit-scrollbar {
              display: none;
            }
            
            /* Ensure content div can scroll */
            div[style*="margin-top: 16px"] {
              overflow: auto !important;
              max-height: calc(100vh - 100px); /* Adjust based on header/footer heights */
            }
            
            /* Tab transition animations */
            .tab-container {
              width: 100%;
              position: relative;
              overflow: visible; /* Changed from hidden to visible */
            }
            
            .tab-content {
              transition: transform 300ms ease;
              width: 100%;
              overflow: auto; /* Allow content to scroll */
              max-height: calc(100vh - 150px); /* Give space for header and buttons */
            }
            
            .slide-left {
              transform: translateX(-100%);
            }
            
            .slide-right {
              transform: translateX(100%);
            }
            
            .tab-enter {
              position: absolute;
              top: 0;
              width: 100%;
              opacity: 0;
            }
            
            .tab-enter.slide-left {
              right: -100%;
            }
            
            .tab-enter.slide-right {
              left: -100%;
            }
            
            /* Tab title styling */
            .tab-title-container {
              cursor: pointer;
              display: inline-block;
              position: relative;
            }
            
            .tab-title-container:after {
              content: '';
              position: absolute;
              width: 100%;
              transform: scaleX(0);
              height: 1px;
              bottom: -2px;
              left: 0;
              background-color: #52c41a;
              transform-origin: bottom right;
              transition: transform 0.3s ease-out;
            }
            
            .tab-title-container:hover:after {
              transform: scaleX(1);
              transform-origin: bottom left;
            }
          `}
        </style>

        {/* Side Navigation */}
        <div className='side-bar' style={{ 
          width: '2.8rem',
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          paddingTop: '16px',
          position: 'relative',
          height: '100vh'
        }}>
          {/* App Title - Vertical Text */}
          <div className='title-logo' style={{
            position: 'absolute',
            top: '50%',
            left: '-28px',
            transform: 'rotate(-90deg) translateX(-50%)',
            transformOrigin: 'left center',
            width: '100vh',
            textAlign: 'center',
            color: '#444',
            letterSpacing: '2px',
            fontSize: '10px',
            fontWeight: 'bold',
            textTransform: 'uppercase',
            pointerEvents: 'none',
            userSelect: 'none',
            zIndex: 1
          }}>
            FOSSModManager
          </div>
          
          <Button 
            type="text" 
            icon={<AppstoreOutlined />} 
            style={{ 
              color: currentTab === 'reframework' ? '#1890ff' : '#666',
              marginBottom: '16px',
              borderRadius: '50%',
              width: '40px',
              height: '40px',
              display: 'flex',
              justifyContent: 'center',
              alignItems: 'center',
              fontSize: '18px'
            }}
            onClick={() => handleTabChange('reframework')}
          />
          
          {/* <Button 
            type="text" 
            icon={<SkinOutlined />} 
            style={{ 
              color: currentTab === 'skin' ? '#1890ff' : '#666',
              marginBottom: '16px',
              borderRadius: '50%',
              width: '40px',
              height: '40px',
              display: 'flex',
              justifyContent: 'center',
              alignItems: 'center',
              fontSize: '18px'
            }}
            onClick={() => handleTabChange('skin')}
          /> */}
          
          <Button 
            type="text" 
            icon={<SearchOutlined />} 
            style={{ 
              color: currentTab === 'search' ? '#1890ff' : '#666',
              marginBottom: '16px',
              borderRadius: '50%',
              width: '40px',
              height: '40px',
              display: 'flex',
              justifyContent: 'space-evenly',
              alignItems: 'center',
              fontSize: '18px'
            }}
            onClick={() => handleTabChange('search')}
          />
          
          <Button 
            type="text" 
            icon={<SettingOutlined />} 
            style={{ 
              color: currentTab === 'settings' ? '#1890ff' : '#666',
              marginBottom: '16px',
              borderRadius: '50%',
              width: '40px',
              height: '40px',
              display: 'flex',
              justifyContent: 'center',
              alignItems: 'center',
              fontSize: '18px'
            }}
            onClick={() => handleTabChange('settings')}
          />
        </div>
        
        {/* Content Area */}
        <Layout.Content style={{ 
          flex: 1, 
          padding: '16px', 
          background: '#000', 
          position: 'relative',
          overflow: 'auto'
        }}>
          {/* Tab Content */}
          <div style={{ marginTop: '16px', overflow: 'hidden' }}>

            {/* Tab Title Section */}
            <div style={{ 
              display: 'flex', 
              justifyContent: 'center', 
              alignItems: 'center',
              marginBottom: '16px'
            }}>
              <Text style={{ 
                color: '#ddd', 
                fontWeight: 500, 
                fontSize: '16px',
                letterSpacing: '0.5px'
              }}>
                <span 
                  className="tab-title-container"
                  onClick={() => handleTabChange('reframework')}
                  style={{ color: currentTab === 'reframework' ? '#52c41a' : '#ddd' }}
                >
                  REFramework Mods
                </span>
                {' / '}
                <span 
                  className="tab-title-container"
                  onClick={() => handleTabChange('skin')}
                  style={{ color: currentTab === 'skin' ? '#52c41a' : '#ddd' }}
                >
                  Skins
                </span>
              </Text>
            </div>

            {/* Tab Container with Animation and Lazy Loading */}
            <div className="tab-container">
              {renderTabContent()}
            </div>
          </div>
          
          {/* Place installercard at bottom right of screen */}
          {(currentTab === 'reframework' || currentTab === 'skin') && (
            <div style={{ position: 'absolute', bottom: '16px', right: '16px' }}>
              <InstallerCard gameRoot={gameConfig.game_root_path} />
            </div>
          )}
          
          {(currentTab === 'reframework') && (
            <div style={{ position: 'absolute', bottom: '16px', left: '16px' }}>
              <CustomInstallButton 
                onClick={handleInstallModFromZip}
                disabled={isInstalling}
                icon={<DownloadOutlined />}
                emphasized={installedMods.length === 0}
                style={{ minWidth: '220px' }}
              >
                Install Mod(s) from Zip
              </CustomInstallButton>
            </div>
          )}
        </Layout.Content>
      </Layout>
    );
  }

  return null;
};

export default MainContent; 