import React from 'react';
import { Menu } from 'antd';
import {
  AppstoreOutlined, // Game List
  SettingOutlined,  // Settings
  // CodeOutlined, // Removed umu-run for now
} from '@ant-design/icons';

function getItem(label, key, icon, children, type) {
  return {
    key,
    icon,
    children,
    label,
    type,
  };
}

// Simplified menu items
const items = [
  getItem('Game List', '1', <AppstoreOutlined />),
  getItem('Settings', '2', <SettingOutlined />),
];

// Receive collapsed state as a prop
const Sidebar = ({ collapsed }) => { 
  const onClick = (e) => {
    console.log('click ', e);
    // Add navigation logic here later
  };

  return (
    <>
      {/* Simplified: Logo/Title can be handled in Header or here later */}
      <div 
        style={{
          height: '32px',
          margin: '16px',
          background: 'rgba(255, 255, 255, 0.2)',
          borderRadius: '6px',
          textAlign: 'center',
          lineHeight: '32px',
          fontSize: '18px',
          fontWeight: 'bold',
          color: '#fff',
          overflow: 'hidden',
        }}
      >
        {collapsed ? 'FMM' : 'FossModManager'} 
      </div>
      <Menu
        theme="dark"
        onClick={onClick}
        defaultSelectedKeys={['1']}
        mode="inline"
        inlineCollapsed={collapsed} // Control menu collapse based on prop
        items={items}
        style={{ borderRight: 0 }}
      />
    </>
  );
};

export default Sidebar; 