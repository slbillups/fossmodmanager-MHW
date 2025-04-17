import React, { useState } from 'react';
import {
  AppstoreOutlined, // For Game List button
  SettingOutlined,  // For Settings button
  SearchOutlined, // For Search button
} from '@ant-design/icons';
import { Button, Layout, Space } from 'antd'; // Removed Menu
// Sidebar import is no longer needed
import MainContent from './components/MainContent';
import SettingsPage from './components/SettingsPage'; // Import SettingsPage
import SearchPage from './components/SearchPage'; // Import SearchPage
import './AppCustomStyles.css';

const { Content } = Layout;

function App() {
  const [currentView, setCurrentView] = useState('games'); // State for current view

  return (
    <Layout style={{ minHeight: '100vh' }}> {/* Removed relative positioning */}
      {/* Removed floating-nav div */}

      <Layout className="site-layout">
        <Content
          style={{
            // Use default padding or adjust via CSS
            // margin: '16px',
            // padding: 24,
            minHeight: 280,
            position: 'relative', // Keep for potential absolute elements inside if needed
          }}
        >
          {/* Top Toolbar Area */}
          <div className="top-toolbar">
            <Space className="top-toolbar-left">
              <Button type="text" icon={<AppstoreOutlined />} onClick={() => setCurrentView('games')}>
                Games
              </Button>
              <Button type="text" icon={<SettingOutlined />} onClick={() => setCurrentView('settings')}>
                Settings
              </Button>
              <Button type="text" icon={< SearchOutlined />} onClick={() => setCurrentView('search')}>
                 Search
              </Button>
            </Space>
            <Space className="top-toolbar-right">
              {/* Maybe add other global actions here later */}
              {/* Conditionally render the update button */}
              {currentView === 'games' && (
                <Button type="primary">
                  Check for updates
                </Button>
              )}
            </Space>
          </div>

          {/* Main Table Content / Settings Page / Search Page */}
          {currentView === 'games' && <MainContent />}
          {currentView === 'settings' && <SettingsPage />}
          {currentView === 'search' && <SearchPage />} {/* Render SearchPage */}
        </Content>
      </Layout>

      {/* Removed bottom-bar div */}
    </Layout>
  );
}

export default App;
