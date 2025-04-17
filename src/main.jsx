import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
// import "./App.css"; // Remove this line
import "./AppCustomStyles.css";

import { ConfigProvider, theme } from "antd"; // Import ConfigProvider and theme
import "antd/dist/reset.css"; // Import Ant Design CSS reset


ReactDOM.createRoot(document.getElementById("root")).render(
  <React.StrictMode>
    <ConfigProvider
      theme={{
        algorithm: theme.darkAlgorithm,
      }}
    >
      <App />
    </ConfigProvider>
  </React.StrictMode>,
);
