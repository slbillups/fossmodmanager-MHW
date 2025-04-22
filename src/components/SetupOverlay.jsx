import React from 'react';
import { Button, notification } from 'antd';
import { open } from '@tauri-apps/plugin-dialog'; // Correct import
import { invoke } from '@tauri-apps/api/core';

const SetupOverlay = ({ onSetupComplete }) => {

  const handleSetup = async () => {
    try {
      const selectedPath = await open({
        multiple: false,
        directory: false, // Only allow selecting files
        title: 'Select Game Executable (e.g., MHWilds.exe)',
        // Add filters if needed, e.g., [{ name: 'Executable', extensions: ['exe'] }]
      });

      if (selectedPath && typeof selectedPath === 'string') {
        console.log('Selected executable:', selectedPath);

        // Step 1: Validate the installation
        const validatedData = await invoke('validate_game_installation', { executablePath: selectedPath });
        console.log('Validation successful:', validatedData);

        // Step 2: Call the callback with the validated data
        // The parent component will handle saving the config.
        onSetupComplete(validatedData);

        // Keep notification for user feedback
        notification.success({
          message: 'Game Path Validated',
          description: 'Game location confirmed. Saving configuration...',
          duration: 2
        });

      } else {
        console.log('No file selected or dialog cancelled.');
      }
    } catch (error) {
      console.error('Error during setup validation:', error);
      const errorMessage = typeof error === 'string' ? error : 'Failed to validate game path. Check console for details.';
      notification.error({ message: 'Setup Error', description: errorMessage });
    }
  };

  return (
    // ... existing JSX structure ...
          <Button type="text" onClick={handleSetup} className="setup-start-button">
             {/* ... existing button content ... */}
          </Button>
    // ... existing JSX structure ...
  );
};

export default SetupOverlay; 