import React, { createContext, useState, useEffect, useCallback, useContext } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';

// Create the context
export const GameConfigContext = createContext(null);

// Create a provider component
export const GameConfigProvider = ({ children }) => {
  const [gameConfig, setGameConfig] = useState(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState(null);

  // Combined initial fetch function
  const initializeApp = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    console.log("[Context] Initializing or Re-initializing app state...");
    let needsSetupResult = true; // Default to true
    try {
      // 1. Get startup state (needs_setup flag)
      console.log("[Context] Fetching startup state...");
      const startupInfo = await invoke('get_startup_state');
      needsSetupResult = startupInfo.needs_setup;
      console.log("[Context] Received startup info:", { needsSetupResult });

      // 2. If setup is NOT needed, load the config
      if (!needsSetupResult) {
        console.log("[Context] Setup not needed, fetching game config...");
        const config = await invoke('load_game_config');
        console.log("[Context] Loaded config:", config);
        setGameConfig(config);
      } else {
        console.log("[Context] Setup needed, ensuring config is null.");
        setGameConfig(null); // Ensure config is null if setup is needed
      }

    } catch (err) {
      console.error('[Context] Error during initialization:', err);
      setError(`Failed to initialize: ${err}`);
      setGameConfig(null);
    } finally {
      setIsLoading(false);
      console.log("[Context] Initialization attempt complete. isLoading=false"); 
    }
  }, []);

  // Call initializeApp on initial mount
  useEffect(() => {
    initializeApp();
  }, []);

  // Listen for the event indicating setup completion and config save
  useEffect(() => {
    let unlisten = null;
    const setupListener = async () => {
      try {
        const currentWindow = getCurrentWindow();
        console.log(`[Context] Setting up listener on window: ${currentWindow.label}`);
        unlisten = await currentWindow.listen('config-saved-and-ready', (event) => {
          console.log("[Context] Received 'config-saved-and-ready' event:", event);
          console.log("[Context] Re-initializing app state after setup completion...");
          initializeApp();
        });
        console.log("[Context] Listener attached for 'config-saved-and-ready'.");
      } catch (error) {
        console.error("[Context] Failed to attach window listener for 'config-saved-and-ready':", error);
      }
    };

    setupListener();

    return () => {
      if (unlisten) {
        unlisten();
        console.log("[Context] Listener detached for 'config-saved-and-ready'.");
      }
    };
  }, []);

  const handleSetupComplete = useCallback(async (newConfig) => {
    console.log("[Context] handleSetupComplete called (likely deprecated for initial setup):", newConfig);
    setGameConfig(newConfig);
    console.log("[Context] Context manually updated.");
  }, []);

  const value = {
    gameConfig,
    isLoading,
    error,
    initializeApp,
    handleSetupComplete,
  };

  return (
    <GameConfigContext.Provider value={value}>
      {children}
    </GameConfigContext.Provider>
  );
};

export const useGameConfig = () => {
  const context = useContext(GameConfigContext);
  if (context === undefined) {
    throw new Error('useGameConfig must be used within a GameConfigProvider');
  }
   if (context === null) {
     return { gameConfig: null, isLoading: true, error: null, initializeApp: () => Promise.resolve(), handleSetupComplete: () => Promise.resolve() };
   }
  return context;
}; 