import React from 'react'; // Removed useState
// Restore imports
import {
  AppstoreOutlined, // For Game List button
  SettingOutlined,  // For Settings button
  SearchOutlined, // For Search button
} from '@ant-design/icons';
import { Button, Layout, Space } from 'antd'; // Removed Menu
import { Link, Outlet } from 'react-router-dom'; // Import Link and Outlet
/* // Keep component imports commented for now
// Sidebar import is no longer needed
import LoadInitial from './components/LoadInitial';
*/
// Remove direct component imports, they will be handled by the router
// import MainContent from './components/MainContent'; 
// import SettingsPage from './components/SettingsPage'; 
// import SearchPage from './components/SearchPage'; 
import './AppCustomStyles.css';

const { Content } = Layout; // Restore Content destructuring

function App() {
  // const [currentView, setCurrentView] = useState('games'); // Remove state

  // Use Link for navigation, Outlet for rendering content
  return (
    <Layout style={{ minHeight: '100vh' }}>
      <Layout className="site-layout">
        <Content
          style={{
            minHeight: 280,
            position: 'relative',
          }}
        >
          {/* Router Outlet: Content is rendered here based on the route */}
          <Outlet />

          {/* Removed conditional rendering block */}
          {/* {currentView === 'games' && <MainContent />} */}
          {/* {currentView === 'settings' && <SettingsPage />} */}
          {/* {currentView === 'search' && <SearchPage />} */}
        </Content>
      </Layout>
    </Layout>
  );
}

export default App;
