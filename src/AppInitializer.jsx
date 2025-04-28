import React, { Suspense, useEffect } from 'react';
import { RouterProvider } from 'react-router-dom';
import { Spin, Alert } from 'antd';
import { useGameConfig } from './contexts/GameConfigContext';
// SetupOverlay is no longer rendered here
// import SetupOverlay from './components/SetupOverlay'; 
// invoke and getCurrentWindow might not be needed directly anymore unless for other purposes
// import { invoke } from '@tauri-apps/api/core';
// import { getCurrentWindow } from '@tauri-apps/api/window';

// Loading component
const LoadingFallback = () => (
  <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100vh' }}>
    <Spin size="large" tip="Initializing..." />
  </div>
);

const AppInitializer = ({ router }) => {
  // We still need the context, but maybe not handleSetupComplete
  const { isLoading, error, gameConfig, initializeApp } = useGameConfig(); 
  // Removed needsSetup and handleSetupComplete from destructuring
  console.log(`AppInitializer render: isLoading=${isLoading}, error=${error}, gameConfig=`, gameConfig); 

  // Remove the specific handler for SetupOverlay completion
  // const handleOverlaySetupComplete = async (validatedData) => { ... };

  // Remove the useEffect that tried to show the window explicitly
  // It should be shown by the backend listener for 'setup-complete'
  // useEffect(() => { ... }, [isLoading, error, needsSetup]); 

  // Initial check on mount to ensure context attempts initialization
  // (Though the provider likely handles this already)
  // useEffect(() => {
  //   initializeApp(); 
  // }, [initializeApp]);

  // --- Simplified Render Logic --- 

  // Still loading the initial state OR waiting for setup to complete
  if (isLoading || !gameConfig) { // Show loading if context is loading OR if config isn't available yet
    console.log("AppInitializer: Rendering LoadingFallback (waiting for context/config)...", {isLoading, hasGameConfig: !!gameConfig});
    return <LoadingFallback />;
  }

  // An error occurred during context initialization (e.g., failed to load config)
  if (error) {
    console.log("AppInitializer: Rendering Error Alert...", {error});
    return (
      <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100vh', padding: '20px' }}>
        <Alert message="Initialization Error" description={error} type="error" showIcon />
        <button onClick={initializeApp}>Retry Initialization</button> {/* Add a retry */} 
      </div>
    );
  }

  // If loading is done, no error, and gameConfig IS available, render the main app router
  console.log("AppInitializer: Rendering RouterProvider...", {gameConfig});
  return <RouterProvider router={router} />;
};

export default AppInitializer; 