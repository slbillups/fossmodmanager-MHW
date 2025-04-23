import React, { useState, useEffect } from 'react';
import { List, Card, Spin, Typography, Tag, notification } from 'antd';
import { invoke } from '@tauri-apps/api/core';
import { ReloadOutlined } from '@ant-design/icons';

const { Title, Text } = Typography;
const { Meta } = Card;

const SkinMods = ({ gameRoot }) => {
  const [skinMods, setSkinMods] = useState([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [imageData, setImageData] = useState({});

  // Fetch installed skin mods by scanning for modinfo.ini files
  const fetchSkinMods = async () => {
    if (!gameRoot) return;
    
    setLoading(true);
    setError(null);
    
    try {
      // Call the Rust command to scan for skin mods
      const mods = await invoke('scan_for_skin_mods', { gameRootPath: gameRoot });
      console.log('Found skin mods:', mods);
      setSkinMods(mods || []);
      
      // Load images for each mod
      const imageDataMap = {};
      for (const mod of mods || []) {
        if (mod.screenshot_path) {
          try {
            // Use the invoke method to read the image through a Rust command
            // This will respect the permissions defined in the capabilities
            const imgData = await invoke('read_mod_image', { 
              imagePath: mod.screenshot_path 
            });
            
            if (imgData) {
              imageDataMap[mod.screenshot_path] = `data:image/png;base64,${imgData}`;
            }
          } catch (imgErr) {
            console.error('Error loading image:', mod.screenshot_path, imgErr);
          }
        }
      }
      setImageData(imageDataMap);
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

  useEffect(() => {
    fetchSkinMods();
  }, [gameRoot]);

  return (
    <div style={{ padding: '0 24px 24px' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 16 }}>
        <Title level={4}>Active Skins</Title>
        <div>
          <ReloadOutlined 
            onClick={fetchSkinMods} 
            style={{ fontSize: 24, cursor: 'pointer' }}
            spin={loading}
          />
        </div>
      </div>
      
      {error && (
        <div style={{ marginBottom: 16, padding: 16, background: '#fff1f0', border: '1px solid #ffa39e', borderRadius: 4 }}>
          <Text type="danger">{error}</Text>
        </div>
      )}
      
      {loading ? (
        <div style={{ textAlign: 'center', padding: 40 }}>
          <Spin size="large" />
        </div>
      ) : (
        <List
          grid={{ gutter: 16, xs: 1, sm: 2, md: 3, lg: 4, xl: 5, xxl: 6 }}
          dataSource={skinMods}
          locale={{ emptyText: 'No skin mods found. Extract game assets and add skin mods to the fossmodmanager/mods directory.' }}
          renderItem={(mod) => (
            <List.Item>
              <Card
                hoverable
                cover={mod.screenshot_path && imageData[mod.screenshot_path] ? (
                  <div style={{ height: 200, position: 'relative' }}>
                    <img 
                      alt={mod.name || 'Mod Screenshot'} 
                      src={imageData[mod.screenshot_path]}
                      style={{ 
                        height: '100%', 
                        width: '100%', 
                        objectFit: 'cover',
                        display: 'block'
                      }} 
                      onError={(e) => {
                        console.error('Image failed to load:', mod.screenshot_path);
                        e.target.onerror = null;
                        e.target.src = 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=';
                      }}
                    />
                  </div>
                ) : null}
              >
                <Meta 
                  title={mod.name || 'Unnamed Mod'} 
                  description={
                    <>
                      {mod.description && <div>{mod.description}</div>}
                      {mod.author && <div>By: {mod.author}</div>}
                      {mod.version && <div>Version: {mod.version}</div>}
                    </>
                  }
                />
                <div style={{ marginTop: 12 }}>
                  <Tag color="blue">{mod.path.split('/').pop()}</Tag>
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
