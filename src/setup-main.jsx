import React from 'react';
import ReactDOM from 'react-dom/client';
import SetupOverlay from './components/SetupOverlay'; // Assuming SetupOverlay handles UI and logic
import { emit } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import './components/styles.css'; // Import global styles if needed

// This is the main entry point for the setup window

const SetupApp = () => {

  // Callback function passed to SetupOverlay
  const handleSetupComplete = async (validatedData) => {
    console.log("Setup window: handleSetupComplete called with", validatedData);
    if (!validatedData) {
      console.error("Setup window: Setup completed without valid data.");
      // Handle error state in SetupOverlay if needed
      return; // Don't proceed if data is invalid
    }

    try {
      // 1. Save the configuration
      console.log("Setup window: Saving game config...", validatedData);
      await invoke('save_game_config', { gameData: validatedData });
      console.log("Setup window: Game config saved.");

      // 2. Emit event to Rust backend to signal completion
      console.log("Setup window: Emitting setup-complete event...");
      await emit('setup-complete');
      console.log("Setup window: setup-complete event emitted.");

      // Note: Rust backend will handle closing this window and showing the main one.
    } catch (err) {
      console.error("Setup window: Error during final setup steps:", err);
      // Update UI in SetupOverlay to show this error
      // Maybe use notification API? Or pass error state back to SetupOverlay.
      // For now, just log it.
    }
  };

  // Render the SetupOverlay component, passing the completion handler
  return <SetupOverlay onSetupComplete={handleSetupComplete} />;
};


ReactDOM.createRoot(document.getElementById('root')).render(
  <React.StrictMode>
    <SetupApp />
  </React.StrictMode>
); 