import React, { useState, useEffect } from 'react';

// Helper function to load the mod database
async function loadModDatabase() {
  try {
    // Assuming you placed mods.json in public/ or src/assets/
    // Adjust the path as needed depending on your project setup and build tool.
    // If it's in src/ and not automatically copied to the output build dir,
    // you might need to import it directly if your build tool supports it,
    // or ensure it's copied during the build process.
    // Let's assume it's accessible via fetch at '/mods.json' relative to the base URL.
    const response = await fetch('/src/data/mods.json'); // Adjust path if needed
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    const modData = await response.json();
    console.log("Mod database loaded:", modData);
    return modData;
  } catch (error) {
    console.error('Error loading mod database:', error);
    return null; // Return null or an empty object/array on error
  }
}

function LocalMods({ selectedGameAppId }) { // Assume you pass the selected game's AppID as a prop
  const [modDatabase, setModDatabase] = useState(null);
  const [gameMods, setGameMods] = useState([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState(null);

  // Effect to load the entire database once on mount
  useEffect(() => {
    setIsLoading(true);
    loadModDatabase()
      .then(data => {
        setModDatabase(data);
        setError(null); // Clear previous errors
      })
      .catch(err => { // Catch potential errors from the async function itself if needed
        console.error("Caught error setting mod database state:", err);
        setError("Failed to load mod database.");
        setModDatabase(null);
      })
      .finally(() => {
        setIsLoading(false);
      });
  }, []); // Empty dependency array means run only once on mount

  // Effect to filter mods when the database or selected game changes
  useEffect(() => {
    if (modDatabase && selectedGameAppId) {
      // Find the entry for the current game using the AppID
      const gameData = modDatabase[selectedGameAppId];
      if (gameData && gameData.mods) {
        setGameMods(gameData.mods);
      } else {
        setGameMods([]); // No mods found for this AppID
      }
    } else {
      setGameMods([]); // Reset if no database or no game selected
    }
  }, [modDatabase, selectedGameAppId]); // Re-run when database or selected game changes

  // --- Render logic ---
  if (isLoading) {
    return <div>Loading mod database...</div>;
  }

  if (error) {
    return <div style={{ color: 'red' }}>Error: {error}</div>;
  }

  if (!selectedGameAppId) {
      return <div>Please select a game to see available mods.</div>;
  }

  return (
    <div>
      <h2>Available Mods for Game {selectedGameAppId}</h2>
      {gameMods.length > 0 ? (
        <ul>
          {gameMods.map(mod => (
            <li key={mod.id}>
              <strong>{mod.name}</strong> by {mod.author} (v{mod.version})
              <p>{mod.description_short}</p>
              {/* Add button or link here to show full details using mod.nexus_url */}
            </li>
          ))}
        </ul>
      ) : (
        <p>No mods listed for this game in the database.</p>
      )}
    </div>
  );
}

export default LocalMods;