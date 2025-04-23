// src-tauri/hooks.js
import fs from 'node:fs';
import path from 'node:path';

module.exports = function (ctx) {
  if (ctx.configuration.tauri.build?.beforeDevCommand) {
    // Check environment
    const isDev = process.env.TAURI_ENV === 'development';
    
    // Path to tauri.conf.json
    const configPath = path.join(__dirname, 'tauri.conf.json');
    const config = require(configPath);
    
    if (isDev) {
      // Add development capability
      if (!config.app.security.capabilities.includes("dev-capability")) {
        config.app.security.capabilities.push("dev-capability");
      }
    } else {
      // Remove development capability in strict mode
      config.app.security.capabilities = config.app.security.capabilities
        .filter(cap => cap !== "dev-capability");
    }
    
    // Write modified configuration
    fs.writeFileSync(configPath, JSON.stringify(config, null, 2));
  }
};