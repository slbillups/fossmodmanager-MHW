import React from 'react';
import { Result, Button } from 'antd';
import { useNavigate } from 'react-router-dom';

const ErrorPage = () => {
  const navigate = useNavigate();

  return (
    <Result
      status="error"
      title="Something Went Wrong"
      subTitle="An unexpected error occurred. Please try again or return to the home page."
      extra={[
        <Button type="primary" key="home" onClick={() => navigate('/')}>
          Back Home
        </Button>,
        <Button key="reload" onClick={() => window.location.reload()}>
          Reload Page
        </Button>,
      ]}
    />
  );
};

export default ErrorPage;
