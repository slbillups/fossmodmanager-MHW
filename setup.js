import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

// Updated element IDs
const browseButton = document.getElementById('browseButton');
const selectedPathDisplay = document.getElementById('selectedPathDisplay');
const finalizeButton = document.getElementById('finalizeButton');
const errorDiv = document.getElementById('error');

// Variable to store the selected path
let selectedExecutablePath = null;

// No need to load paths automatically anymore
// async function loadPaths() { ... }

// --- Event Listener for Browse Button ---
async function handleBrowse() {
    errorDiv.style.display = 'none'; // Hide previous errors
    try {
        const selected = await open({
            multiple: false,
            directory: false,
            // Optional: Add filters for common executable types
            // filters: [
            //     { name: 'Executable', extensions: ['exe'] }, // Windows
            //     { name: 'All Files', extensions: ['*'] } // Linux/Mac (often no extension)
            // ]
        });

        if (selected) {
            // selected is a single path string if not multiple
            selectedExecutablePath = selected; // Use selected directly
            selectedPathDisplay.textContent = selectedExecutablePath;
            selectedPathDisplay.style.fontStyle = 'normal'; // Make text normal
            selectedPathDisplay.style.color = '#000';
            finalizeButton.disabled = false; // Enable finalize button
        } else {
            // User cancelled the dialog
            // Keep finalize button disabled if nothing was previously selected
            if (!selectedExecutablePath) {
                 finalizeButton.disabled = true;
            }
        }
    } catch (err) {
        console.error('Error opening file dialog:', err);
        errorDiv.textContent = `Error opening dialog: ${err}`;
        errorDiv.style.display = 'block';
        selectedExecutablePath = null; // Reset path on error
        selectedPathDisplay.textContent = 'Error selecting file...';
        selectedPathDisplay.style.fontStyle = 'italic';
        selectedPathDisplay.style.color = 'red';
        finalizeButton.disabled = true;
    }
}

// --- Event Listener for Finalize Button ---
async function finalize() {
    if (!selectedExecutablePath) {
        errorDiv.textContent = 'Please select a game executable first.';
        errorDiv.style.display = 'block';
        return;
    }

    console.log('Finalizing setup with executable path:', selectedExecutablePath);
    errorDiv.style.display = 'none';
    finalizeButton.disabled = true; // Disable while processing
    finalizeButton.textContent = 'Processing...';
    browseButton.disabled = true; // Disable browse button too

    try {
        // Call the backend command with the executable path
        // Note the change in the argument name to executablePath
        await invoke('finalize_setup', { executablePath: selectedExecutablePath });
        // The backend will close this window upon success
    } catch (err) {
        console.error('Error finalizing setup:', err);
        errorDiv.textContent = `Error: ${err}`;
        errorDiv.style.display = 'block';
        finalizeButton.disabled = false; // Re-enable on error
        browseButton.disabled = false;
        finalizeButton.textContent = 'Finalize Setup';
    }
}

// --- Attach Event Listeners ---
if (browseButton) {
    browseButton.addEventListener('click', handleBrowse);
} else {
    console.error('Could not find browse button.');
}

if (finalizeButton) {
    finalizeButton.addEventListener('click', finalize);
} else {
    console.error('Could not find finalize button.');
}

// Remove the DOMContentLoaded listener that called loadPaths
// document.addEventListener('DOMContentLoaded', loadPaths); 