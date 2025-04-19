import React from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter, Routes, Route } from "react-router-dom"; // Import router components
import App from "./App";
// import "./App.css"; // Remove this line
import "./AppCustomStyles.css";

// Import page components for routing
import MainContent from "./components/MainContent";
import SettingsPage from "./components/SettingsPage";
import SearchPage from "./components/SearchPage";

import { ConfigProvider, theme } from "antd"; // Import ConfigProvider and theme
// import "antd/dist/reset.css"; // Import Ant Design CSS reset


ReactDOM.createRoot(document.getElementById("root")).render(
  <React.StrictMode>
    {/* Wrap with BrowserRouter */}
    <BrowserRouter>
      <ConfigProvider
        theme={{
          algorithm: theme.darkAlgorithm,
        }}
      >
        {/* Define Routes - App provides layout via Outlet */}
        <Routes>
          <Route path="/" element={<App />}>
            {/* Nested routes render inside App's Outlet */}
            <Route index element={<MainContent />} /> 
            <Route path="settings" element={<SettingsPage />} />
            <Route path="search" element={<SearchPage />} />
            {/* Add other routes here as needed */}
          </Route>
        </Routes>
      </ConfigProvider>
    </BrowserRouter>
  </React.StrictMode>,
);
