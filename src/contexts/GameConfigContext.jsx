import React, { createContext, useState, useEffect, useCallback, useContext } from 'react';
import { invoke } from '@tauri-apps/api/core';

// Create the context
export const GameConfigContext = createContext(null);

// Create a provider component
export const GameConfigProvider = ({ children }) => {
  const [gameConfig, setGameConfig] = useState(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState(null);

  const fetchGameConfig = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    console.log("[Context] Attempting to invoke load_game_config...");
    try {
      const config = await invoke('load_game_config');
      console.log("[Context] Loaded config:", config);
      setGameConfig(config);
    } catch (err) {
      console.error('[Context] Error loading game config:', err);
      setError(`Failed to load configuration: ${err}`);
      setGameConfig(null);
    } finally {
      setIsLoading(false);
    }
  }, []);

  // Load config on initial mount
  useEffect(() => {
    fetchGameConfig();
  }, [fetchGameConfig]);

  // Value provided to consuming components
  const value = {
    gameConfig,
    isLoading,
    error,
    fetchGameConfig, // Provide the function to reload/reset
  };

  return (
    <GameConfigContext.Provider value={value}>
      {children}
    </GameConfigContext.Provider>
  );
};

// Create a custom hook for easy consumption
export const useGameConfig = () => {
  const context = useContext(GameConfigContext);
  if (context === undefined) {
    throw new Error('useGameConfig must be used within a GameConfigProvider');
  }
   if (context === null) {
     // This can happen initially before the provider sets the value
     // Depending on strictness, you might return null or throw an error
     // Let's return null for now, components should handle loading/null states
     return { gameConfig: null, isLoading: true, error: null, fetchGameConfig: () => Promise.resolve() };
   }
  return context;
}; 