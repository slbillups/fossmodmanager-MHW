import { useState, useEffect, useCallback, useContext } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { notification } from 'antd';
import { GameConfigContext } from '../contexts/GameConfigContext'; // Assuming context provides gameConfig

export const useInstaller = () => {
    const { gameConfig } = useContext(GameConfigContext); // Get gameConfig from context
    const [isRfInstalled, setIsRfInstalled] = useState(null); // null: unknown, true: installed, false: not installed
    const [isLoadingCheck, setIsLoadingCheck] = useState(false);
    const [isLoadingInstall, setIsLoadingInstall] = useState(false);
    const [error, setError] = useState(null);

    const checkStatus = useCallback(async () => {
        if (!gameConfig?.game_root_path) {
            setIsRfInstalled(null); // Reset if no config
            return;
        }

        setIsLoadingCheck(true);
        setError(null);
        try {
            console.log('useInstaller: Checking REFramework status...');
            const installed = await invoke('check_reframework_installed', {
                gameRootPath: gameConfig.game_root_path,
            });
            setIsRfInstalled(installed);
            console.log('useInstaller: REFramework status:', installed);
        } catch (err) {
            console.error('Error checking REFramework status:', err);
            setError(`Failed to check REFramework status: ${err}`);
            setIsRfInstalled(null); // Set to unknown on error
            notification.error({
                message: 'REFramework Check Failed',
                description: typeof err === 'string' ? err : 'Could not determine REFramework status.',
            });
        } finally {
            setIsLoadingCheck(false);
        }
    }, [gameConfig]);

    const triggerRfInstall = useCallback(async () => {
        if (!gameConfig?.game_root_path) {
            setError('Cannot install REFramework: Game path not configured.');
            notification.error({ message: 'Install Error', description: 'Game path not configured.' });
            return;
        }

        setIsLoadingInstall(true);
        setError(null);
        notification.info({ 
            message: 'REFramework Installation Started', 
            description: 'Downloading and installing REFramework... This may take a moment.', 
            key: 'rfInstall', 
            duration: 0 // Keep open until success/error
        });

        try {
            console.log('useInstaller: Triggering REFramework installation...');
            await invoke('ensure_reframework', {
                gameRootPath: gameConfig.game_root_path,
                // Pass appHandle if the backend command needs it (currently doesn't)
            });
            setIsRfInstalled(true); // Assume success means installed
            console.log('useInstaller: REFramework installation successful.');
            notification.success({
                key: 'rfInstall',
                message: 'REFramework Installed',
                description: 'REFramework has been successfully installed.',
            });
            // Optionally re-check status after install for confirmation?
            // checkStatus(); 
        } catch (err) {
            console.error('Error installing REFramework:', err);
            const errorMsg = typeof err === 'string' ? err : 'Installation failed. Check logs.';
            setError(`Failed to install REFramework: ${errorMsg}`);
            notification.error({
                key: 'rfInstall',
                message: 'REFramework Installation Failed',
                description: errorMsg,
            });
            // Status remains unknown or potentially false after failed install
            // setIsRfInstalled(false); // Or keep null?
        } finally {
            setIsLoadingInstall(false);
        }
    }, [gameConfig]);

    // Effect to check status when gameConfig changes
    useEffect(() => {
        console.log('useInstaller useEffect: gameConfig changed, checking status.', gameConfig);
        checkStatus();
    }, [checkStatus]); // Depend on the memoized checkStatus

    return {
        isRfInstalled,
        isLoading: isLoadingCheck || isLoadingInstall, // Combined loading state
        isLoadingCheck,
        isLoadingInstall,
        error,
        checkStatus,      // Expose check function if manual refresh is needed
        triggerRfInstall, // Expose install function
    };
}; 