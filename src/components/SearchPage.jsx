import React, { useState, useEffect } from 'react';
import { Select, Card, List, Space, Row, Col, Typography, Spin } from 'antd';
import { AppstoreOutlined, BarsOutlined } from '@ant-design/icons'; // Example icons
import { invoke } from '@tauri-apps/api/core'; // Added invoke import
import { open } from '@tauri-apps/plugin-shell'; // <-- Import the open function

const { Title, Text } = Typography;
const { Meta } = Card;

// Dummy data - replace with actual game data and API calls
const userGames = [
  { value: 'darksouls', label: 'Dark Souls' }, // Assuming value is domain name
  { value: 'darksouls2', label: 'Dark Souls II' }, // Assuming value is domain name
  { value: 'eldenring', label: 'Elden Ring' }, // Assuming value is domain name
  { value: 'sekiro', label: 'Sekiro: Shadows Die Twice' }, // Assuming value is domain name
  { value: 'monsterhunterwilds', label: 'Monster Hunter Wilds' }, // Added for testing
];

// Dummy mod data - replace with API call results (kept for fallback maybe? or remove)
// const dummyMods = { ... }; // We might remove this later

const sortOptions = [
  { value: 'default', label: 'Default' },
  { value: 'popular', label: 'Most Popular' },
  { value: 'newest', label: 'Newest' },
  { value: 'endorsed', label: 'Most Endorsed' },
];

const sortOrderOptions = [
  { value: 'desc', label: 'Descending' },
  { value: 'asc', label: 'Ascending' },
];

function SearchPage() {
  const [selectedGame, setSelectedGame] = useState(null);
  const [sortBy, setSortBy] = useState('default');
  const [sortOrder, setSortOrder] = useState('desc');
  const [mods, setMods] = useState([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null); // Add error state

  // Fetch mods from backend when selection changes
  useEffect(() => {
    if (selectedGame) {
      setLoading(true);
      setError(null); // Clear previous errors
      setMods([]); // Clear previous mods

      console.log(`Invoking fetch_trending_mods for game: ${selectedGame}`);

      invoke('fetch_trending_mods', { gameDomainName: selectedGame })
        .then(fetchedMods => {
          console.log('Fetched mods:', fetchedMods);
          // Map backend data (NexusMod) to frontend structure
          const mappedMods = fetchedMods.map(mod => ({
            id: mod.mod_id.toString(), // Ensure ID is string if needed by List key
            title: mod.name,
            description: mod.summary || 'No description available.', // Provide fallback
            // Use picture_url, fallback to a placeholder if missing
            imageUrl: mod.picture_url || `https://via.placeholder.com/150/808080/FFFFFF?text=${encodeURIComponent(mod.name)}`,
            // Add other fields if needed, e.g., from mod.version, mod.author etc.
          }));
          setMods(mappedMods);
        })
        .catch(err => {
          console.error('Error fetching trending mods:', err);
          setError(`Failed to fetch mods: ${err}`);
        })
        .finally(() => {
          setLoading(false);
        });

    } else {
      setMods([]); // Clear mods if no game is selected
      setError(null); // Clear error if no game is selected
    }
    // We don't need sortBy or sortOrder in dependency array yet as V1 trending doesn't use them
  }, [selectedGame]);

  const handleGameChange = (value) => {
    setSelectedGame(value);
  };

  const handleSortByChange = (value) => {
    setSortBy(value);
  };

  const handleSortOrderChange = (value) => {
    setSortOrder(value);
  };

  // Filter options for Select component
  const filterOption = (input, option) =>
    (option?.label ?? '').toLowerCase().includes(input.toLowerCase());

  // Function to handle clicking a mod card
  const handleCardClick = async (mod) => {
    if (!selectedGame) {
      console.error("Cannot open mod page without a selected game.");
      // Optionally: Show a user-facing error message here
      return;
    }
    const modUrl = `https://www.nexusmods.com/${selectedGame}/mods/${mod.id}`;
    console.log(`Opening URL: ${modUrl}`);
    try {
      await open(modUrl); // Use the imported open function
    } catch (error) {
      console.error("Failed to open URL:", error);
      // Optionally: Show a user-facing error message here
      setError(`Failed to open mod page: ${error}`); // Display error in the UI
    }
  };

  return (
    <div style={{ padding: '24px' }}>
      <Title level={3}>Search Mods</Title>

      {/* Selection/Filter Row */}
      <Row gutter={[16, 16]} style={{ marginBottom: '24px' }} align="bottom">
        <Col xs={24} sm={12} md={8}>
          <Text>Select Game:</Text>
          <Select
            showSearch
            placeholder="Select a game"
            optionFilterProp="children"
            onChange={handleGameChange}
            filterOption={filterOption}
            options={userGames}
            style={{ width: '100%' }}
            allowClear
          />
        </Col>
        <Col xs={24} sm={12} md={8}>
          <Text>Sort By:</Text>
          <Select
            defaultValue="default"
            onChange={handleSortByChange}
            options={sortOptions}
            style={{ width: '100%' }}
            disabled={!selectedGame} // Disable until a game is selected
          />
        </Col>
        <Col xs={24} sm={12} md={8}>
           <Text>Order:</Text>
          <Select
            defaultValue="desc"
            onChange={handleSortOrderChange}
            options={sortOrderOptions}
            style={{ width: '100%' }}
            disabled={!selectedGame || sortBy === 'default'} // Also disable if sort is default
          />
        </Col>
      </Row>

      {/* Mod Results Area */}
      {loading ? (
        <div style={{ textAlign: 'center', padding: '50px' }}>
          <Spin size="large" />
        </div>
      ) : error ? ( // Display error message if an error occurred
        <div style={{ textAlign: 'center', padding: '50px', color: 'red' }}>
          <Text type="danger">Error: {error}</Text>
        </div>
      ) : selectedGame ? (
        <List
          grid={{ gutter: 16, xs: 1, sm: 2, md: 3, lg: 4, xl: 5, xxl: 6 }} // Responsive grid
          dataSource={mods}
          renderItem={(mod) => (
            <List.Item key={mod.id}> {/* Added key prop */}
              <Card
                hoverable
                cover={<img alt={mod.title} src={mod.imageUrl} style={{height: 150, objectFit: 'cover'}} />}
                onClick={() => handleCardClick(mod)} // <-- Add onClick handler here
                // Add actions like download/view details later
                // actions={[
                //   <SettingOutlined key="setting" />,
                //   <EditOutlined key="edit" />,
                //   <EllipsisOutlined key="ellipsis" />,
                // ]}
              >
                <Meta title={mod.title} description={mod.description} />
              </Card>
            </List.Item>
          )}
           locale={{ emptyText: mods.length === 0 && !loading ? 'No trending mods found for this game.' : ' ' }} // Improved empty text
        />
      ) : (
         <div style={{ textAlign: 'center', padding: '50px', color: 'rgba(255, 255, 255, 0.45)' }}>
           <Text>Please select a game to search for mods.</Text>
         </div>
      )}
    </div>
  );
}

export default SearchPage; 