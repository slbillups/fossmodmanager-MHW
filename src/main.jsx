import React from "react";
import ReactDOM from "react-dom/client";
import { createBrowserRouter, RouterProvider } from "react-router-dom";
import { BrowserRouter, Routes, Route } from "react-router-dom"; // Import router components
import App from "./App";
// import "./App.css"; // Remove this line
import "./AppCustomStyles.css";

// Import page components for routing
import MainContent from "./components/MainContent";
import SettingsPage from "./components/SettingsPage";
import SearchPage from "./components/SearchPage";
import ErrorPage from "./ErrorPage";

import { ConfigProvider, theme, App as AntApp } from "antd"; // Import ConfigProvider, theme, and App as AntApp to avoid naming conflict
// import "antd/dist/reset.css"; // Import Ant Design CSS reset
import { GameConfigProvider } from "./contexts/GameConfigContext"; // Import the provider

// Define routes
const router = createBrowserRouter([
  {
    path: "/",
    element: <App />, // App likely contains the main layout (header)
    errorElement: <ErrorPage />,
    children: [
      {
        index: true, // Default child route
        element: <MainContent /> // Render MainContent in the App's Outlet
      },
      {
        path: "search",
        element: <SearchPage />
      },
      {
        path: "settings",
        element: <SettingsPage /> // SettingsPage also renders in the App's Outlet
      },
      // Add other child routes here if needed
    ]
  },
  // Add other top-level routes if necessary (e.g., a dedicated setup page outside App layout)
]);

ReactDOM.createRoot(document.getElementById("root")).render(
  <React.StrictMode>
    <GameConfigProvider>
      <ConfigProvider
        theme={{
          algorithm: theme.darkAlgorithm,
        }}
      >
        {/* Wrap RouterProvider with AntApp */}
        <AntApp>
          <RouterProvider router={router} />
        </AntApp>
      </ConfigProvider>
    </GameConfigProvider>
  </React.StrictMode>,
);
