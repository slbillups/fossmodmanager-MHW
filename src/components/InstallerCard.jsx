import React from 'react';
import { Card, Button, Spin, Alert, Typography, Tag } from 'antd';
import { CheckCircleOutlined, CloseCircleOutlined, DownloadOutlined, SyncOutlined } from '@ant-design/icons';
import { useInstaller } from '../hooks/useInstaller';

const { Text } = Typography;

const InstallerCard = () => {
    const { 
        isRfInstalled, 
        isLoading, 
        isLoadingCheck, 
        isLoadingInstall, 
        error,
        triggerRfInstall 
    } = useInstaller();

    let statusContent;
    if (isLoadingCheck) {
        statusContent = (
            <>
                <Spin size="small" />
                <Text type="secondary" style={{ marginLeft: 8 }}>Checking REFramework status...</Text>
            </>
        );
    } else if (isRfInstalled === true) {
        statusContent = (
            <Tag icon={<CheckCircleOutlined />} color="success">
                REFramework Installed
            </Tag>
        );
    } else if (isRfInstalled === false) {
        statusContent = (
            <Tag icon={<CloseCircleOutlined />} color="warning">
                REFramework Not Found
            </Tag>
        );
    } else { // null or other states
        statusContent = (
             <Tag color="default">Status Unknown</Tag>
        );
    }

    return (
        <Card 
            title="REFramework Status"
            size="small"
            style={{ marginBottom: '16px' }} // Add some spacing
            extra={statusContent}
        >
            {error && (
                <Alert 
                    message="Error" 
                    description={error} 
                    type="error" 
                    showIcon 
                    style={{ marginBottom: '12px' }} 
                />
            )}

            {isRfInstalled === false && !isLoadingInstall && (
                <>
                    <Text>REFramework is required for most mods to function.</Text>
                    <Button 
                        type="primary" 
                        icon={<DownloadOutlined />} 
                        onClick={triggerRfInstall}
                        loading={isLoadingInstall}
                        style={{ marginTop: '8px' }}
                    >
                        Install REFramework
                    </Button>
                </>
            )}

            {isLoadingInstall && (
                <>
                    <Spin />
                    <Text type="secondary" style={{ marginLeft: 8 }}>Installing REFramework...</Text>
                </>
            )}

            {/* Optionally add a manual refresh button? */}
            {/* 
            <Button 
                icon={<SyncOutlined />} 
                onClick={checkStatus} 
                loading={isLoadingCheck}
                disabled={isLoadingInstall}
                size="small"
                style={{ float: 'right' }}
            >
                Refresh Status
            </Button> 
            */}
        </Card>
    );
};

export default InstallerCard; 