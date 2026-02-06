import React, { useState, useEffect, useRef } from 'react';
import { List, Card, Spin, Typography, Tag, notification, Button, Switch, Tooltip, Popconfirm } from 'antd';
import { invoke } from '@tauri-apps/api/core';
import { ReloadOutlined, CheckCircleOutlined, StopOutlined, DeleteOutlined } from '@ant-design/icons';
import LoadingOverlay from './LoadingOverlay';

const { Title, Text } = Typography;
const { Meta } = Card;

const SkinMods = ({ gameRoot }) => {
  const [skinMods, setSkinMods] = useState([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [imageData, setImageData] = useState({});
  const [processingMods, setProcessingMods] = useState(new Set());
  const cachedImageRefs = useRef({});
  const [processingDeleteSkin, setProcessingDeleteSkin] = useState(new Set());

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
    const modPath = mod.path;
    setProcessingMods(prev => new Set(prev).add(modPath));

    // --- Optimistic UI Update --- 
    const originalMods = [...skinMods]; // Backup for revert
    setSkinMods(prevMods =>
      prevMods.map(m =>
        m.path === modPath ? { ...m, base: { ...m.base, enabled: enable } } : m
      )
    );
    // --- End Optimistic UI Update ---

    try {
      // Call the appropriate function based on the toggle action
      if (enable) {
        await invoke('enable_skin_mod_via_registry', { 
          gameRootPath: gameRoot,
          modPath: modPath
        });
      } else {
        await invoke('disable_skin_mod_via_registry', { 
          gameRootPath: gameRoot,
          modPath: modPath
        });
      }
      
      notification.success({
        message: `Skin ${enable ? 'Enabled' : 'Disabled'}`,
        description: `Successfully ${enable ? 'enabled' : 'disabled'} ${mod.name}`
      });
      
      // Refresh the mod list to show updated status
      fetchSkinMods();
    } catch (err) {
      console.error(`Error ${actionType.toLowerCase()} skin mod:`, err);
      notification.error({
        message: `${actionType} Error`,
        description: typeof err === 'string' ? err : `Failed to ${actionType.toLowerCase()} skin mod`
      });
      // --- Revert Optimistic Update on Error ---
      setSkinMods(originalMods);
      fetchSkinMods(); // Also refresh on error to be sure
      // --- End Revert ---
    } finally {
      setProcessingMods(prev => {
        const next = new Set(prev);
        next.delete(modPath);
        return next;
      });
    }
  };

  // --- Delete Handler for Skin Mods ---
  const handleDeleteSkinMod = async (mod) => {
    if (!gameRoot) {
      notification.error({ message: 'Error', description: 'Game root directory not set' });
      return;
    }

    const modPath = mod.path; // Identifier is the original path
    const modName = mod.name || mod.directory_name; // For messages

    setProcessingDeleteSkin(prev => new Set(prev).add(modPath));

    try {
      await invoke('delete_skin_mod', {
        gameRootPath: gameRoot,
        modPath: modPath,
      });

      notification.success({
        message: 'Skin Deleted',
        description: `Successfully deleted skin mod '${modName}'.`,
      });

      // Refresh the list
      fetchSkinMods();

    } catch (err) {
      console.error(`Error deleting skin mod ${modName}:`, err);
      notification.error({
        message: 'Deletion Error',
        description: typeof err === 'string' ? err : `Failed to delete skin mod '${modName}'`,
      });
      // Optionally refresh even on error to sync state
      fetchSkinMods(); 
    } finally {
      setProcessingDeleteSkin(prev => {
        const next = new Set(prev);
        next.delete(modPath);
        return next;
      });
    }
  };
  // --- End Delete Handler ---

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
      {/* Use the new LoadingOverlay component */}
      <LoadingOverlay isLoading={processingMods.size > 0} tip="Updating Skin(s)..." />

      {/* Add custom styles to override Ant Design defaults */}
      <style>{`
        .skin-mod-card .ant-card-actions {
          background: transparent !important;
          border-top: none !important; /* Also remove the top border often associated with actions */
        }

        /* Customize Switch appearance */
        .skin-mod-card .ant-switch {
          background-color: #f5222d !important; /* Red background for unchecked/disabled */
        }

        .skin-mod-card .ant-switch-checked {
          background-color: #52c41a !important; /* Green background for checked/enabled */
        }

        /* Ensure handle is always visible (white) */
        .skin-mod-card .ant-switch .ant-switch-handle::before {
          background-color: #ffffff !important;
        }
      `}</style>

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
          renderItem={(mod) => {
            return (
            <List.Item>
              <Card className="skin-mod-card"
                hoverable
                style={{ background: 'transparent', border: 'none' }}
                cover={(
                  <div style={{ height: 200, position: 'relative' }}>
                    {mod.thumbnail_path && imageData[mod.thumbnail_path] ? (
                      <img 
                        alt={mod.name || 'Mod Screenshot'}
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
                    {/* --- Container for Top-Right Elements --- */}
                    <div style={{
                      position: 'absolute',
                      top: 8,
                      right: 8,
                      display: 'flex',
                      alignItems: 'center',
                      gap: '8px' // Add space between badge and button
                    }}>
                      {/* --- Status Badge (Now inside the container) --- */}
                      <div 
                        style={{ 
                          // Removed absolute positioning, handled by parent
                          background: mod.enabled ? 'rgba(82, 196, 26, 0.8)' : 'rgba(245, 34, 45, 0.8)',
                          color: 'white',
                          padding: '2px 8px',
                          borderRadius: '4px',
                          display: 'flex',
                          alignItems: 'center',
                          cursor: 'pointer'
                        }}
                        onClick={(e) => {
                          e.stopPropagation();
                          toggleModEnabled(mod, !mod.enabled);
                        }}
                      >
                        {mod.enabled ? 
                          <><CheckCircleOutlined style={{ marginRight: 5 }} /> Enabled</> : 
                          <><StopOutlined style={{ marginRight: 5 }} /> Disabled</>
                        }
                      </div>
                      {/* --- Delete Button Popconfirm (Moved Here) --- */}
                      <Popconfirm
                        title={`Delete skin '${mod.name || mod.directory_name}'?`}
                        description="This removes the mod files from the manager and game (if enabled). This cannot be undone."
                        onConfirm={() => handleDeleteSkinMod(mod)}
                        okText="Yes, Delete"
                        cancelText="Cancel"
                        okButtonProps={{ danger: true }}
                        // Prevent click event from bubbling up to the badge toggle
                        onClick={(e) => e.stopPropagation()}
                      >
                        <Button
                          type="text"
                          icon={<DeleteOutlined />}
                          size="small"
                          loading={processingDeleteSkin.has(mod.path)}
                          danger
                          // Add a subtle background for better visibility
                          style={{ color: '#ff4d4f', backgroundColor: 'rgba(0, 0, 0, 0.5)', borderRadius: '4px'}}
                        />
                      </Popconfirm>
                    </div>
                    {/* --- End Container --- */}
                  </div>
                )}
              >
                <Meta 
                  title={
                    <div style={{ 
                      whiteSpace: 'normal',
                      overflowWrap: 'break-word',
                      textTransform: 'capitalize' 
                    }}>
                      {mod.name || 'Unnamed Mod'}
                    </div>
                  } 
                  description={
                    <>
                      {mod.description ? (
                        <div style={{ marginTop: 4 }}>{mod.description}</div>
                      ) : (
                        <div style={{ marginTop: 4, fontStyle: 'italic' }}>No additional information available</div>
                      )}
                      {mod.author && <div style={{ marginTop: 4 }}>By: {mod.author}</div>}
                      {mod.version && <div style={{ marginTop: 4 }}>Version: {mod.version}</div>}
                    </>
                  }
                />
              </Card>
            </List.Item>
            );
          }}
        />
      )}
    </div>
  );
};

export default SkinMods;
