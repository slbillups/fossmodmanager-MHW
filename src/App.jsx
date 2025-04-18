import React, { useState } from 'react';
// Restore imports
import {
  AppstoreOutlined, // For Game List button
  SettingOutlined,  // For Settings button
  SearchOutlined, // For Search button
} from '@ant-design/icons';
import { Button, Layout, Space } from 'antd'; // Removed Menu
/* // Keep component imports commented for now
// Sidebar import is no longer needed
import LoadInitial from './components/LoadInitial';
*/
import MainContent from './components/MainContent'; // Restore MainContent import
// Restore other component imports
import SettingsPage from './components/SettingsPage'; // Import SettingsPage
import SearchPage from './components/SearchPage'; // Import SearchPage
import './AppCustomStyles.css';

const { Content } = Layout; // Restore Content destructuring

function App() {
  const [currentView, setCurrentView] = useState('games'); // Restore state

  // Restore original layout but without rendering main content components yet
  return (
    <Layout style={{ minHeight: '100vh' }}>
      <Layout className="site-layout">
        <Content
          style={{
            minHeight: 280,
            position: 'relative',
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
            <Space className="top-toolbar-right" />
          </div>

          {/* Main Table Content / Settings Page / Search Page - Keep commented */}
          {currentView === 'games' && <MainContent />} {/* Restore MainContent rendering */}
          {currentView === 'settings' && <SettingsPage />} {/* Restore SettingsPage rendering */}
          {currentView === 'search' && <SearchPage />} {/* Restore SearchPage rendering */}
        </Content>
      </Layout>
    </Layout>
  );
}

export default App;
