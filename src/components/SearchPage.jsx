import React, { useState, useEffect } from 'react';
import { Select, Card, List, Space, Row, Col, Typography, Spin, Input } from 'antd';
import { AppstoreOutlined, BarsOutlined, SearchOutlined } from '@ant-design/icons';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-shell';

const { Title, Text } = Typography;
const { Meta } = Card;
const { Search } = Input;

// Sort options for the dropdown
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
  const [mods, setMods] = useState([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const [sortBy, setSortBy] = useState('default');
  const [sortOrder, setSortOrder] = useState('desc');
  const [searchQuery, setSearchQuery] = useState('');
  const [viewMode, setViewMode] = useState('grid'); // 'grid' or 'list'
  const [page, setPage] = useState(1);

  // Fetch mods from Nexus API
  const fetchMods = async () => {
    setLoading(true);
    setError(null);
    
    try {
      // Hardcoded for Monster Hunter Wilds
      const response = await invoke('fetch_trending_mods', { 
        gameDomainName: "monsterhunterwilds",
        page,
        sortBy: sortBy === 'default' ? null : sortBy,
        sortOrder
      });
      
      setMods(response || []);
    } catch (err) {
      console.error('Error fetching mods:', err);
      setError(typeof err === 'string' ? err : 'Failed to load mods');
    } finally {
      setLoading(false);
    }
  };

  // Fetch mods when sort options change
  useEffect(() => {
    fetchMods();
  }, [page, sortBy, sortOrder]);

  // Handle opening a mod on Nexus Mods
  const handleOpenMod = async (mod) => {
    try {
      // Construct Nexus Mods URL from mod ID
      const url = `https://www.nexusmods.com/monsterhunterwilds/mods/${mod.mod_id}`;
      await open(url);
    } catch (err) {
      console.error('Failed to open URL:', err);
    }
  };

  // Filter mods by search query
  const filteredMods = mods.filter(mod => 
    searchQuery ? 
      mod.name.toLowerCase().includes(searchQuery.toLowerCase()) || 
      (mod.summary && mod.summary.toLowerCase().includes(searchQuery.toLowerCase()))
    : true
  );

  return (
    <div style={{ padding: '0 24px 24px' }}>
      <Title level={4}>Monster Hunter Wilds Mods</Title>
      
      {/* Search and filter controls */}
      <Row gutter={[16, 16]} style={{ marginBottom: 16 }}>
        <Col xs={24} md={8}>
          <Search
            placeholder="Search mods..."
            allowClear
            enterButton={<SearchOutlined />}
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
          />
        </Col>
        <Col xs={12} md={6}>
          <Select
            style={{ width: '100%' }}
            placeholder="Sort by"
            value={sortBy}
            onChange={setSortBy}
            options={sortOptions}
          />
        </Col>
        <Col xs={12} md={6}>
          <Select
            style={{ width: '100%' }}
            placeholder="Order"
            value={sortOrder}
            onChange={setSortOrder}
            options={sortOrderOptions}
          />
        </Col>
        <Col xs={24} md={4}>
          <Space>
            <AppstoreOutlined 
              onClick={() => setViewMode('grid')}
              style={{ fontSize: 24, color: viewMode === 'grid' ? '#1890ff' : undefined, cursor: 'pointer' }}
            />
            <BarsOutlined 
              onClick={() => setViewMode('list')}
              style={{ fontSize: 24, color: viewMode === 'list' ? '#1890ff' : undefined, cursor: 'pointer' }}
            />
            <a onClick={fetchMods}>Refresh</a>
          </Space>
        </Col>
      </Row>
      
      {/* Display error if any */}
      {error && (
        <div style={{ marginBottom: 16, padding: 16, background: '#fff1f0', border: '1px solid #ffa39e', borderRadius: 4 }}>
          <Text type="danger">{error}</Text>
        </div>
      )}
      
      {/* Loading indicator */}
      {loading && (
        <div style={{ textAlign: 'center', padding: 40 }}>
          <Spin size="large" />
        </div>
      )}
      
      {/* Mods display */}
      {!loading && (
        viewMode === 'grid' ? (
          <List
            grid={{ gutter: 16, xs: 1, sm: 2, md: 3, lg: 4, xl: 5, xxl: 6 }}
            dataSource={filteredMods}
            pagination={{
              onChange: (page) => setPage(page),
              pageSize: 20,
              defaultCurrent: 1,
              total: filteredMods.length,
            }}
            renderItem={(mod) => (
              <List.Item>
                <Card
                  hoverable
                  cover={mod.picture_url ? <img alt={mod.name} src={mod.picture_url} /> : null}
                  onClick={() => handleOpenMod(mod)}
                >
                  <Meta 
                    title={mod.name} 
                    description={
                      <>
                        <div>Downloads: {mod.total_downloads ? mod.total_downloads.toLocaleString() : 'N/A'}</div>
                        <div>Endorsements: {mod.endorsements_count ? mod.endorsements_count.toLocaleString() : 'N/A'}</div>
                      </>
                    } 
                  />
                </Card>
              </List.Item>
            )}
          />
        ) : (
          <List
            itemLayout="horizontal"
            dataSource={filteredMods}
            pagination={{
              onChange: (page) => setPage(page),
              pageSize: 10,
              defaultCurrent: 1,
              total: filteredMods.length,
            }}
            renderItem={(mod) => (
              <List.Item 
                actions={[
                  <a onClick={() => handleOpenMod(mod)}>View on Nexus</a>
                ]}
              >
                <List.Item.Meta
                  avatar={mod.picture_url ? <img alt={mod.name} src={mod.picture_url} style={{ width: 80, height: 80, objectFit: 'cover' }} /> : null}
                  title={mod.name}
                  description={
                    <>
                      <div>{mod.summary}</div>
                      <div>Downloads: {mod.total_downloads ? mod.total_downloads.toLocaleString() : 'N/A'} | Endorsements: {mod.endorsements_count ? mod.endorsements_count.toLocaleString() : 'N/A'}</div>
                    </>
                  }
                />
              </List.Item>
            )}
          />
        )
      )}
    </div>
  );
}

export default SearchPage; 