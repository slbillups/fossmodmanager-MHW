import React, { useEffect } from 'react';
import { Spin } from 'antd';

const LoadingOverlay = ({ isLoading, tip = "Processing..." }) => {
  useEffect(() => {
    if (isLoading) {
      // Apply wait cursor to the whole page
      document.body.style.cursor = 'wait';
    } else {
      // Remove wait cursor when not loading
      document.body.style.cursor = '';
    }

    // Cleanup function to ensure cursor is reset if component unmounts while loading
    return () => {
      document.body.style.cursor = '';
    };
  }, [isLoading]); // Rerun effect when isLoading changes

  if (!isLoading) {
    return null;
  }

  return (
    <div style={{
      position: 'fixed', // Cover the whole viewport
      top: 0,
      left: 0,
      width: '100vw',
      height: '100vh',
      background: 'rgba(0, 0, 0, 0.6)', // Semi-transparent background
      display: 'flex',
      justifyContent: 'center',
      alignItems: 'center',
      zIndex: 1000, // Ensure it's on top
    }}>
      <Spin size="large" tip={tip} />
    </div>
  );
};

export default LoadingOverlay; 