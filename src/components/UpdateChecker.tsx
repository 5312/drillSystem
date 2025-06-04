import { useState, useEffect } from 'react';
import { Button, message, Modal, Space } from 'antd';
import { invoke } from '@tauri-apps/api/core';
import { relaunch } from '@tauri-apps/plugin-process';
import { listen } from '@tauri-apps/api/event';
import { getVersion } from '@tauri-apps/api/app';

interface UpdatePayload {
  version?: string;
  body?: string;
}

export function UpdateChecker() {
  const [checking, setChecking] = useState(false);
  const [updateAvailable, setUpdateAvailable] = useState(false);
  const [installing, setInstalling] = useState(false);
  const [progress, setProgress] = useState(0);
  const [currentVersion, setCurrentVersion] = useState('');

  // 组件加载时获取当前版本并自动检查更新
  useEffect(() => {
    const init = async () => {
      try {
        const version = await getVersion();
        setCurrentVersion(version);
      } catch (error) {
        console.error('获取版本信息失败:', error);
      }
      
      checkForUpdates();
      
      // 监听更新事件
      const unlisten = await listen<UpdatePayload>('tauri://update-available', (event) => {
        setUpdateAvailable(true);
        Modal.confirm({
          title: '发现新版本',
          content: `有新版本可用: ${event.payload?.version || '未知版本'}\n${event.payload?.body || ''}`,
          okText: '立即更新',
          cancelText: '稍后再说',
          onOk: () => installUpdate(),
        });
      });
      
      return () => {
        unlisten();
      };
    };
    
    init();
  }, []);

  // 检查更新
  const checkForUpdates = async () => {
    setChecking(true);
    try {
      const result = await invoke<string>('check_update');
      if (result.includes('有更新可用')) {
        setUpdateAvailable(true);
      } else {
        message.success('当前已是最新版本');
      }
    } catch (error: any) {
      console.error('检查更新失败:', error);
      message.error(`检查更新失败: ${error.message || error}`);
    } finally {
      setChecking(false);
    }
  };

  // 安装更新
  const installUpdate = async () => {
    setInstalling(true);
    setProgress(0);
    
    try {
      // 监听下载进度
      const unlisten = await listen<{chunkLength?: number; contentLength?: number}>('tauri://update-download-progress', (event) => {
        if (event.payload) {
          const { chunkLength, contentLength } = event.payload;
          if (contentLength && contentLength > 0 && chunkLength) {
            setProgress(Math.round((chunkLength / contentLength) * 100));
          }
        }
      });
      
      // 监听更新就绪
      const unlistenReady = await listen('tauri://update-ready', async () => {
        message.success('更新已下载，即将重启应用');
        // 重启应用以安装更新
        await relaunch();
      });
      
      // 监听更新错误
      const unlistenError = await listen<string>('tauri://update-error', (event) => {
        message.error(`更新失败: ${event.payload || '未知错误'}`);
        setInstalling(false);
        unlistenError();
        unlistenReady();
        unlisten();
      });
      
      // 开始安装更新
      await invoke('install_update');
    } catch (error: any) {
      console.error('安装更新失败:', error);
      message.error(`安装更新失败: ${error.message || error}`);
      setInstalling(false);
    }
  };

  return (
    <Space>
      <span style={{ color: 'white' }}>当前版本: {currentVersion}</span>
      <Button 
        type="primary" 
        onClick={checkForUpdates} 
        loading={checking}
        disabled={installing}
      >
        检查更新
      </Button>
      {updateAvailable && !installing && (
        <Button 
          type="default" 
          onClick={installUpdate}
        >
          安装更新
        </Button>
      )}
      {installing && (
        <span style={{ color: 'white' }}>正在下载更新: {progress}%</span>
      )}
    </Space>
  );
} 