import React, { useState } from 'react';
import { Button, message, Spin, Modal } from 'antd';
import { invoke } from '@tauri-apps/api/core';
import { DatabaseOutlined } from '@ant-design/icons';

const ExtractGameAssets = ({ gameRoot }) => {
  const [extracting, setExtracting] = useState(false);
  const [showModal, setShowModal] = useState(false);

  const handleExtractAssets = async () => {
    if (!gameRoot) {
      message.error('Game root path not configured');
      return;
    }
    
    try {
      setShowModal(true);
      setExtracting(true);
      
      await invoke('extract_game_assets', { 
        gameRootPath: gameRoot 
      });
      
      message.success('Game assets extracted successfully');
    } catch (error) {
      console.error('Error extracting game assets:', error);
      message.error(typeof error === 'string' ? error : 'Failed to extract game assets');
    } finally {
      setExtracting(false);
    }
  };

  return (
    <>
      <Button 
        type="primary" 
        icon={<DatabaseOutlined />}
        onClick={handleExtractAssets}
        disabled={!gameRoot}
      >
        Extract Game Assets
      </Button>
      
      <Modal
        title="Extracting Game Assets"
        open={showModal}
        onCancel={() => !extracting && setShowModal(false)}
        footer={null}
        closable={!extracting}
        maskClosable={!extracting}
      >
        {extracting ? (
          <div style={{ textAlign: 'center', padding: '20px' }}>
            <Spin size="large" />
            <p style={{ marginTop: '16px' }}>
              Extracting game assets. This may take several minutes...
            </p>
          </div>
        ) : (
          <div style={{ textAlign: 'center', padding: '20px' }}>
            <p>Game assets extracted successfully to:</p>
            <code>{`${gameRoot}/fossmodmanager/extracted`}</code>
            <div style={{ marginTop: '16px' }}>
              <Button type="primary" onClick={() => setShowModal(false)}>
                Close
              </Button>
            </div>
          </div>
        )}
      </Modal>
    </>
  );
};

export default ExtractGameAssets; 