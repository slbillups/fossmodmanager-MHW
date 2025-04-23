import React from 'react';
import { Button, Spin, Typography, Tooltip } from 'antd';
import { CheckCircleOutlined, CloseCircleOutlined, DownloadOutlined } from '@ant-design/icons';
import { useInstaller } from '../hooks/useInstaller';

const { Text } = Typography;

const InstallerCard = ({ gameRoot }) => {
    const { 
        isRfInstalled, 
        isLoading, 
        isLoadingCheck, 
        isLoadingInstall, 
        error,
        triggerRfInstall 
    } = useInstaller();

    const getStatusIcon = () => {
        if (isLoadingCheck) {
            return <Spin size="small" style={{ marginRight: 8 }} />;
        } else if (isRfInstalled === true) {
            return <CheckCircleOutlined style={{ color: '#52c41a' }} />;
        } else if (isRfInstalled === false) {
            return <CloseCircleOutlined style={{ color: '#faad14' }} />;
        }
        return null;
    };

    const getStatusText = () => {
        if (isRfInstalled === true) return "Installed";
        if (isRfInstalled === false) return "Not Found";
        return "Unknown";
    };

    return (
        <div style={{ 
            background: 'transparent',
            borderRadius: '4px',
            boxShadow: '0 1px 3px rgba(0, 0, 0, 0.2)',
            overflow: 'hidden'
        }}>
            <div style={{ 
                display: 'flex', 
                alignItems: 'center',
                justifyContent: 'space-between',
                padding: '12px 16px',
                borderBottom: isRfInstalled === false ? '1px solid rgba(250, 173, 20, 0.2)' : 'none'
            }}>
                <div style={{ display: 'flex', alignItems: 'center' }}>
                    <Text strong style={{ 
                        color: '#ddd', 
                        marginRight: 12, 
                        fontSize: '14px',
                        letterSpacing: '0.3px'
                    }}>REFramework</Text>
                    <div style={{ 
                        display: 'flex', 
                        alignItems: 'center',
                        opacity: 0.9,
                    }}>
                        {getStatusIcon()}
                        <Text style={{ 
                            marginLeft: 4, 
                            color: isRfInstalled ? '#7bef41' : '#faad14',
                            fontSize: '13px'
                        }}>
                            {getStatusText()}
                        </Text>
                    </div>
                </div>
                
                <div>
                    {error && (
                        <Tooltip title={error}>
                            <Text type="danger" style={{ marginRight: 12, fontSize: '13px' }}>Error</Text>
                        </Tooltip>
                    )}

                    {isRfInstalled === false && !isLoadingInstall && (
                        <Tooltip title="REFramework is required for most mods to function">
                            <Button 
                                type="text" 
                                style={{
                                    background: 'rgba(82, 196, 26, 0.15)',
                                    border: 'none',
                                    color: '#7bef41',
                                    height: '28px',
                                    padding: '0 12px'
                                }}
                                size="small"
                                icon={<DownloadOutlined />} 
                                onClick={triggerRfInstall}
                                loading={isLoadingInstall}
                            >
                                Install
                            </Button>
                        </Tooltip>
                    )}

                    {isLoadingInstall && (
                        <Text type="secondary" style={{ marginLeft: 8, fontSize: '13px' }}>
                            <Spin size="small" style={{ marginRight: 8 }} />
                            Installing...
                        </Text>
                    )}
                </div>
            </div>
            
            {gameRoot && !isRfInstalled && (
                <div style={{ 
                    padding: '8px 16px', 
                    background: 'rgba(0, 0, 0, 0.3)',
                    display: 'flex',
                    alignItems: 'center'
                }}>
                    <div style={{ display: 'flex', alignItems: 'center', width: '100%' }}>
                        <CloseCircleOutlined style={{ color: '#ff4d4f', marginRight: '8px' }} />
                        <Text style={{ 
                            color: '#ff7875', 
                            fontSize: '13px',
                            cursor: 'pointer'
                        }} onClick={triggerRfInstall}>
                            You haven't installed REFramework yet. Install now?
                        </Text>
                    </div>
                </div>
            )}
        </div>
    );
};

export default InstallerCard; 