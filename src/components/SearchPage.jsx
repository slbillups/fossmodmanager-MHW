import React, { useState, useEffect } from 'react';
import { Select, Card, List, Space, Row, Col, Typography, Spin } from 'antd';
import { AppstoreOutlined, BarsOutlined } from '@ant-design/icons'; // Example icons

const { Title, Text } = Typography;
const { Meta } = Card;

// Dummy data - replace with actual game data and API calls
const userGames = [
  { value: 'ds1', label: 'Dark Souls' },
  { value: 'ds2', label: 'Dark Souls II' },
  { value: 'er', label: 'Elden Ring' },
  { value: 'sekiro', label: 'Sekiro: Shadows Die Twice' },
];

// Dummy mod data - replace with API call results
const dummyMods = {
  ds1: [
    { id: 'mod1', title: 'DS1 Texture Overhaul', description: 'High-resolution textures for Dark Souls.', imageUrl: 'https://via.placeholder.com/150/FF0000/FFFFFF?text=Mod1' },
    { id: 'mod2', title: 'Better Combat Mod', description: 'Improves combat mechanics.', imageUrl: 'https://via.placeholder.com/150/00FF00/000000?text=Mod2' },
  ],
  er: [
    { id: 'mod3', title: 'Elden Ring Seamless Co-op', description: 'Play Elden Ring co-op seamlessly.', imageUrl: 'https://via.placeholder.com/150/0000FF/FFFFFF?text=Mod3' },
    { id: 'mod4', title: 'Performance Boost', description: 'Increases FPS.', imageUrl: 'https://via.placeholder.com/150/FFFF00/000000?text=Mod4' },
    { id: 'mod5', title: 'New Weapons Pack', description: 'Adds 50 new weapons.', imageUrl: 'https://via.placeholder.com/150/FF00FF/FFFFFF?text=Mod5' },
  ],
  // Add more dummy data for other games as needed
};

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

  // Simulate API call when selection changes
  useEffect(() => {
    if (selectedGame) {
      setLoading(true);
      // Simulate network delay
      setTimeout(() => {
        // Fetch mods based on selectedGame, sortBy, sortOrder (implement actual logic later)
        console.log(`Fetching mods for ${selectedGame}, sort: ${sortBy} ${sortOrder}`);
        const gameMods = dummyMods[selectedGame] || [];
        // Add dummy sorting logic here if needed for demo
        setMods(gameMods);
        setLoading(false);
      }, 500);
    } else {
      setMods([]); // Clear mods if no game is selected
    }
  }, [selectedGame, sortBy, sortOrder]);

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
      ) : selectedGame ? (
        <List
          grid={{ gutter: 16, xs: 1, sm: 2, md: 3, lg: 4, xl: 5, xxl: 6 }} // Responsive grid
          dataSource={mods}
          renderItem={(mod) => (
            <List.Item>
              <Card
                hoverable
                cover={<img alt={mod.title} src={mod.imageUrl} style={{height: 150, objectFit: 'cover'}} />}
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
           locale={{ emptyText: 'No mods found for this game.' }}
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