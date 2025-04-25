import React, { Suspense, lazy } from "react";
import ReactDOM from "react-dom/client";
import { createBrowserRouter, RouterProvider } from "react-router-dom";
import { BrowserRouter, Routes, Route } from "react-router-dom"; // Import router components
import App from "./App";
// import "./App.css"; // Remove this line
import "./AppCustomStyles.css";

// Lazy load page components for routing
const MainContent = lazy(() => import("./components/MainContent"));
const SettingsPage = lazy(() => import("./components/SettingsPage"));
const SearchPage = lazy(() => import("./components/SearchPage"));
const ErrorPage = lazy(() => import("./ErrorPage"));

import { ConfigProvider, theme, App as AntApp, Spin } from "antd"; // Import ConfigProvider, theme, and App as AntApp to avoid naming conflict
// import "antd/dist/reset.css"; // Import Ant Design CSS reset
import { GameConfigProvider } from "./contexts/GameConfigContext"; // Import the provider

// Loading component for suspense fallback
const LoadingFallback = () => (
  <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100vh' }}>
    <Spin size="large" tip="Loading..." />
  </div>
);

// Define routes
const router = createBrowserRouter([
  {
    path: "/",
    element: <App />, // App likely contains the main layout (header)
    errorElement: <Suspense fallback={<LoadingFallback />}><ErrorPage /></Suspense>,
    children: [
      {
        index: true, // Default child route
        element: <Suspense fallback={<LoadingFallback />}><MainContent /></Suspense> // Render MainContent in the App's Outlet
      },
      {
        path: "search",
        element: <Suspense fallback={<LoadingFallback />}><SearchPage /></Suspense>
      },
      {
        path: "settings",
        element: <Suspense fallback={<LoadingFallback />}><SettingsPage /></Suspense> // SettingsPage also renders in the App's Outlet
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
