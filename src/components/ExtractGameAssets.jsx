import React, { useState } from 'react';
import { Button, message, Spin, Modal, Tooltip } from 'antd';
import { DatabaseOutlined } from '@ant-design/icons';

const ExtractGameAssets = ({ gameRoot }) => {
  // This functionality has been disabled as we've removed the sidecar implementation
  return (
    <Tooltip title="Currently unavailable - Feature is being refactored">
      <Button 
        type="primary" 
        icon={<DatabaseOutlined />}
        disabled={true}
      >
        Extract Game Assets
      </Button>
    </Tooltip>
  );
};

export default ExtractGameAssets; 