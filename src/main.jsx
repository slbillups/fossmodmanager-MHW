import React, { Suspense, lazy } from "react";
import ReactDOM from "react-dom/client";
import { createBrowserRouter, RouterProvider } from "react-router-dom";
import App from "./App";
import AppInitializer from "./AppInitializer";
import "./AppCustomStyles.css";

// Lazy load page components for routing
const MainContent = lazy(() => import("./components/MainContent"));
const SettingsPage = lazy(() => import("./components/SettingsPage"));
const SearchPage = lazy(() => import("./components/SearchPage"));
const ErrorPage = lazy(() => import("./ErrorPage"));

import { ConfigProvider, theme, App as AntApp, Spin } from "antd";
import { GameConfigProvider } from "./contexts/GameConfigContext";

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
    element: <App />,
    errorElement: <Suspense fallback={<LoadingFallback />}><ErrorPage /></Suspense>,
    children: [
      {
        index: true,
        element: <Suspense fallback={<LoadingFallback />}><MainContent /></Suspense>
      },
      {
        path: "search",
        element: <Suspense fallback={<LoadingFallback />}><SearchPage /></Suspense>
      },
      {
        path: "settings",
        element: <Suspense fallback={<LoadingFallback />}><SettingsPage /></Suspense>
      },
    ]
  },
]);

ReactDOM.createRoot(document.getElementById("root")).render(
  <React.StrictMode>
    <GameConfigProvider>
      <ConfigProvider
        theme={{
          algorithm: theme.darkAlgorithm,
        }}
      >
        <AntApp>
          <AppInitializer router={router} />
        </AntApp>
      </ConfigProvider>
    </GameConfigProvider>
  </React.StrictMode>,
);
