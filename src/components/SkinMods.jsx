import React, { useState, useEffect, useRef } from 'react';
import { List, Card, Spin, Typography, Tag, notification, Button, Switch, Tooltip } from 'antd';
import { invoke } from '@tauri-apps/api/core';
import { ReloadOutlined, CheckCircleOutlined, StopOutlined } from '@ant-design/icons';

const { Title, Text } = Typography;
const { Meta } = Card;

const SkinMods = ({ gameRoot }) => {
  const [skinMods, setSkinMods] = useState([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [imageData, setImageData] = useState({});
  const [processingMod, setProcessingMod] = useState(null);
  const cachedImageRefs = useRef({});

  // Fetch skin mods from the registry
  const fetchSkinMods = async () => {
    if (!gameRoot) return;
    
    setLoading(true);
    setError(null);
    
    try {
      // First scan for new mods and update registry
      const mods = await invoke('scan_and_update_skin_mods', { 
        gameRootPath: gameRoot,
        // No appHandle needed from frontend
      });
      
      // Then load all mods from the updated registry
      // This might seem redundant if scan_and_update returns the list,
      // but ensures we always fetch the definitive state from the registry
      const installedMods = await invoke('list_skin_mods_from_registry', {}); 
      
      console.log('Found skin mods:', installedMods);
      setSkinMods(installedMods || []);
      
      // Load images for each mod
      await loadModImages(installedMods || []);
    } catch (err) {
      console.error('Error loading skin mods:', err);
      setError(typeof err === 'string' ? err : 'Failed to load skin mods');
      notification.error({
        message: 'Skin Mods Error',
        description: `Failed to load skin mods: ${typeof err === 'string' ? err : 'Unknown error'}`,
      });
    } finally {
      setLoading(false);
    }
  };

  // Toggle mod enabled state
  const toggleModEnabled = async (mod, enable) => {
    if (!gameRoot) {
      notification.error({
        message: 'Error',
        description: 'Game root directory is not set'
      });
      return;
    }

    const actionType = enable ? 'Enabling' : 'Disabling';
    setProcessingMod(mod.base.path);
    
    try {
      // Call the appropriate function based on the toggle action
      if (enable) {
        await invoke('enable_skin_mod_via_registry', { 
          gameRootPath: gameRoot,
          modPath: mod.base.path // Access path via base
        });
      } else {
        await invoke('disable_skin_mod_via_registry', { 
          gameRootPath: gameRoot, // Pass gameRoot for consistency, though maybe not needed
          modPath: mod.base.path // Access path via base
        });
      }
      
      notification.success({
        message: `Skin ${enable ? 'Enabled' : 'Disabled'}`,
        description: `Successfully ${enable ? 'enabled' : 'disabled'} ${mod.base.name}`
      });
      
      // Refresh the mod list to show updated status
      fetchSkinMods();
    } catch (err) {
      console.error(`Error ${actionType.toLowerCase()} skin mod:`, err);
      notification.error({
        message: `${actionType} Error`,
        description: typeof err === 'string' ? err : `Failed to ${actionType.toLowerCase()} skin mod`
      });
    } finally {
      setProcessingMod(null);
    }
  };

  // Separate function to handle image loading with cache handling
  const loadModImages = async (mods) => {
    // First, check which images we need to load
    const newImages = {};
    const toLoadPaths = [];
    
    for (const mod of mods) {
      if (!mod.thumbnail_path) continue;
      
      // Check if we already have this image in our state cache
      if (imageData[mod.thumbnail_path]) {
        // Use existing data
        newImages[mod.thumbnail_path] = imageData[mod.thumbnail_path];
      } 
      // Check if we have it in our ref cache (persistent across renders)
      else if (cachedImageRefs.current[mod.thumbnail_path]) {
        newImages[mod.thumbnail_path] = cachedImageRefs.current[mod.thumbnail_path];
      }
      // Need to load it
      else {
        toLoadPaths.push(mod.thumbnail_path);
      }
    }
    
    // Skip the loading phase if we have all images cached
    if (toLoadPaths.length === 0) {
      setImageData(newImages);
      return;
    }
    
    // First attempt to read from the cache
    try {
      // Load the missing images from cache
      const cachedImages = await invoke('get_cached_mod_images', { 
        imagePaths: toLoadPaths 
      }).catch(() => ({})); // Fail gracefully if command doesn't exist yet
      
      // Process successfully cached images
      for (const path in cachedImages) {
        if (cachedImages[path]) {
          newImages[path] = `data:image/png;base64,${cachedImages[path]}`;
          cachedImageRefs.current[path] = newImages[path]; // Save to persistent ref
          
          // Remove from the loading list
          const index = toLoadPaths.indexOf(path);
          if (index > -1) {
            toLoadPaths.splice(index, 1);
          }
        }
      }
    } catch (err) {
      console.warn('Cache fetch failed, will load images directly:', err);
      // Continue with direct loading if cache fails
    }
    
    // Now load any remaining images directly
    for (const path of toLoadPaths) {
      try {
        // Use the invoke method to read the image through a Rust command
        const imgData = await invoke('read_mod_image', { imagePath: path });
        
        if (imgData) {
          newImages[path] = `data:image/png;base64,${imgData}`;
          cachedImageRefs.current[path] = newImages[path]; // Save to persistent ref
          
          // Cache the image for future use
          try {
            await invoke('cache_mod_image', { 
              imagePath: path,
              imageData: imgData
            }).catch(() => {}); // Ignore errors if command doesn't exist yet
          } catch (cacheErr) {
            console.warn(`Failed to cache image ${path}:`, cacheErr);
          }
        }
      } catch (imgErr) {
        console.error('Error loading image:', path, imgErr);
      }
    }
    
    // Update state with all images
    setImageData(newImages);
  };
  
  // Load skin mods on initial render and when gameRoot changes
  useEffect(() => {
    fetchSkinMods();
    return () => {
      // Optional clean up can be added here if needed
    };
  }, [gameRoot]);

  return (
    <div style={{ padding: '0 24px 24px' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 16 }}>
        <Title level={4}>Skins</Title>
        <div>
          <ReloadOutlined 
            onClick={fetchSkinMods} 
            style={{ fontSize: 24, cursor: 'pointer', marginRight: 16 }}
            spin={loading}
          />
        </div>
      </div>
      
      {error && (
        <div style={{ marginBottom: 16, padding: 16, background: '#fff1f0', border: '1px solid #ffa39e', borderRadius: 4 }}>
          <Text type="danger">{error}</Text>
        </div>
      )}
      
      {/* Available Skins Section */}
      {loading ? (
        <div style={{ textAlign: 'center', padding: 40 }}>
          <Spin size="large" />
        </div>
      ) : (
        <List
          grid={{ gutter: 16, xs: 1, sm: 2, md: 3, lg: 4, xl: 5, xxl: 6 }}
          dataSource={skinMods}
          locale={{ emptyText: 'No skin mods found. Add skin mods to the fossmodmanager/mods directory.' }}
          renderItem={(mod) => (
            <List.Item>
              <Card
                hoverable
                cover={(
                  <div style={{ height: 200, position: 'relative' }}>
                    {mod.thumbnail_path && imageData[mod.thumbnail_path] ? (
                      <img 
                        alt={mod.base.name || 'Mod Screenshot'} // Access name via base
                        src={imageData[mod.thumbnail_path]}
                        style={{ 
                          height: '100%', 
                          width: '100%', 
                          objectFit: 'cover',
                          display: 'block'
                        }} 
                        onError={(e) => {
                          console.error('Image failed to load:', mod.thumbnail_path);
                          e.target.onerror = null;
                          e.target.src = '/icons/notfound.svg';
                        }}
                      />
                    ) : (
                      <img 
                        alt="No screenshot available" 
                        src="/icons/notfound.svg"
                        style={{ 
                          height: '100%', 
                          width: '100%', 
                          objectFit: 'contain',
                          display: 'block',
                          padding: '20px'
                        }} 
                      />
                    )}
                    <div style={{ 
                      position: 'absolute', 
                      top: 8, 
                      right: 8, 
                      background: mod.base.enabled ? 'rgba(82, 196, 26, 0.8)' : 'rgba(245, 34, 45, 0.8)', // Access enabled via base 
                      color: 'white',
                      padding: '2px 8px',
                      borderRadius: '4px',
                      display: 'flex',
                      alignItems: 'center'
                    }}>
                      {mod.base.enabled ? 
                        <><CheckCircleOutlined style={{ marginRight: 5 }} /> Enabled</> : 
                        <><StopOutlined style={{ marginRight: 5 }} /> Disabled</>
                      }
                    </div>
                  </div>
                )}
              >
                <Meta 
                  title={<span style={{ textTransform: 'capitalize' }}>{mod.base.name || 'Unnamed Mod'}</span>} 
                  description={
                    <>
                      {mod.base.description && <div>{mod.base.description}</div>}
                      {mod.base.author && <div>By: {mod.base.author}</div>}
                      {mod.base.version && <div>Version: {mod.base.version}</div>}
                      {!mod.base.author && !mod.base.version && !mod.base.description && (
                        <div>No additional information available</div>
                      )}
                    </>
                  }
                />
                <div style={{ marginTop: 12, display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                  <Tag color="blue">{mod.base.path.split(/[\\/]/).pop()}</Tag>
                  <Tooltip title={`${mod.base.enabled ? 'Disable' : 'Enable'} this skin mod`}>
                    <Switch
                      checked={mod.base.enabled}
                      loading={processingMod === mod.base.path}
                      onChange={(checked) => toggleModEnabled(mod, checked)}
                    />
                  </Tooltip>
                </div>
              </Card>
            </List.Item>
          )}
        />
      )}
    </div>
  );
};

export default SkinMods;
