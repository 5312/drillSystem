import { useState, useEffect } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import { Table, Modal, Button, Input, Checkbox, Tabs, Space, Popconfirm } from "antd";
import type { ColumnsType } from "antd/es/table";
import "./App.css";

interface LicenseInfo {
  license_id: string;
  customer_name: string;
  customer_email: string;
  issue_date: string;
  expiry_date: string;
  features: string[];
  signature: string;
  machine_code?: string;
}

interface LicenseValidationResult {
  is_valid: boolean;
  info?: LicenseInfo;
  message: string;
}

function App() {
  const [activeTab, setActiveTab] = useState<'generate' | 'validate' | 'history'>('generate');
  
  // 生成许可证状态
  const [machineCode, setMachineCode] = useState("");
  const [customerName, setCustomerName] = useState("");
  const [customerEmail, setCustomerEmail] = useState("");
  const [expiryDays, setExpiryDays] = useState("365");
  const [features, setFeatures] = useState("");
  const [generatedLicense, setGeneratedLicense] = useState("");
  
  // 验证许可证状态
  const [licenseKey, setLicenseKey] = useState("");
  const [validationResult, setValidationResult] = useState<LicenseValidationResult | null>(null);
  const [validationMachineCode, setValidationMachineCode] = useState("");
  
  // 历史许可证
  const [licenseHistory, setLicenseHistory] = useState<LicenseInfo[]>([]);
  
  // 公钥
  const [publicKey, setPublicKey] = useState("");
  const [isPublicKeyModalOpen, setIsPublicKeyModalOpen] = useState(false);
  
  // 加载历史许可证
  useEffect(() => {
    if (activeTab === 'history') {
      loadLicenseHistory();
    }
  }, [activeTab]);
  
  async function loadLicenseHistory() {
    try {
      const licenses = await invoke<LicenseInfo[]>("get_licenses");
      setLicenseHistory(licenses);
    } catch (error) {
      console.error("加载许可证历史失败:", error);
    }
  }
  
  // 获取当前机器码
  async function getCurrentMachineId() {
    try {
      const machineId = await invoke<string>("get_current_machine_id");
      if (activeTab === 'generate') {
        setMachineCode(machineId);
      } else if (activeTab === 'validate') {
        setValidationMachineCode(machineId);
      }
    } catch (error) {
      console.error("获取机器码失败:", error);
      Modal.error({
        title: '获取机器码失败',
        content: `错误: ${error}`
      });
    }
  }
  
  // 生成许可证
  async function generateLicense() {
    try {
      const featuresList = features.split(',').map(f => f.trim()).filter(f => f);
      let license;
      
      // 处理过期时间，0表示永不过期
      const parsedExpiryDays = parseInt(expiryDays);
      const actualExpiryDays = parsedExpiryDays <= 0 ? 0 : parsedExpiryDays;
      
      if (machineCode) {
        license = await invoke<string>("generate_license_key_with_machine_code", { 
          customerName, 
          customerEmail, 
          expiryDays: actualExpiryDays, 
          features: featuresList,
          machineCode
        });
      } else {
        license = await invoke<string>("generate_license_key", { 
          customerName, 
          customerEmail, 
          expiryDays: actualExpiryDays, 
          features: featuresList 
        });
      }
      
      setGeneratedLicense(license);
      // 如果成功生成许可证，清空表单
      setCustomerName("");
      setCustomerEmail("");
      setExpiryDays("365");
      setFeatures("");
    } catch (error) {
      console.error("生成许可证失败:", error);
      setGeneratedLicense(`错误: ${error}`);
    }
  }
  
  // 验证许可证
  async function validateLicense() {
    try {
      let result;
      
      if (validationMachineCode) {
        result = await invoke<LicenseValidationResult>("validate_license_key_with_machine_code", { 
          licenseKey,
          machineCode: validationMachineCode
        });
      } else {
        result = await invoke<LicenseValidationResult>("validate_license_key", { 
          licenseKey 
        });
      }
      
      setValidationResult(result);
    } catch (error) {
      console.error("验证许可证失败:", error);
      setValidationResult({
        is_valid: false,
        message: `错误: ${error}`
      });
    }
  }
  
  // 删除许可证
  async function deleteLicense(licenseId: string) {
    try {
      await invoke("delete_license_by_id", { licenseId });
      // 重新加载许可证列表
      loadLicenseHistory();
      Modal.success({
        content: '许可证已删除'
      });
    } catch (error) {
      console.error("删除许可证失败:", error);
      Modal.error({
        title: '删除失败',
        content: `错误: ${error}`
      });
    }
  }
  
  // 复制到剪贴板
  const copyToClipboard = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
      Modal.success({
        content: '已复制到剪贴板'
      });
    } catch (error) {
      console.error("复制到剪贴板失败:", error);
      Modal.error({
        title: '复制失败',
        content: `${error}`
      });
    }
  };

  // 格式化日期
  const formatDate = (dateString: string) => {
    if (!dateString) return '无日期';
    
    try {
      const date = new Date(dateString);
      // 检查日期是否有效
      if (isNaN(date.getTime())) {
        return '日期无效';
      }
      
      // 检查是否是永不过期的日期（通常是很远的未来日期）
      const now = new Date();
      const yearDiff = date.getFullYear() - now.getFullYear();
      if (yearDiff > 100) {
        return '永不过期';
      }
      
      return date.toLocaleDateString('zh-CN');
    } catch (e) {
      console.error("日期格式化错误:", e);
      return '日期错误';
    }
  };

  // 导出公钥
  async function exportPublicKey() {
    try {
      const key = await invoke<string>("export_license_public_key");
      setPublicKey(key);
      setIsPublicKeyModalOpen(true);
    } catch (error) {
      console.error("导出公钥失败:", error);
      Modal.error({
        title: '导出公钥失败',
        content: `${error}`
      });
    }
  }

  // 表格列定义
  const columns: ColumnsType<LicenseInfo> = [
    {
      title: '客户名称',
      dataIndex: 'customer_name',
      key: 'customer_name',
      width: 150,
    },
    {
      title: '客户邮箱',
      dataIndex: 'customer_email',
      key: 'customer_email',
      width: 200,
    },
    {
      title: '发行日期',
      dataIndex: 'issue_date',
      key: 'issue_date',
      width: 120,
      render: (text) => formatDate(text),
    },
    {
      title: '过期日期',
      dataIndex: 'expiry_date',
      key: 'expiry_date',
      width: 120,
      render: (text) => formatDate(text),
    },
    {
      title: '功能列表',
      dataIndex: 'features',
      key: 'features',
      render: (features) => features.join(', '),
    },
    {
      title: '机器码',
      dataIndex: 'machine_code',
      key: 'machine_code',
      width: 150,
      render: (text) => text || '无限制',
    },
    {
      title: '操作',
      key: 'action',
      width: 200,
      render: (_, record) => (
        <Space size="small">
          <Button type="primary" size="small" onClick={() => exportPublicKey()}>
            查看公钥
          </Button>
          <Popconfirm
            title="确定删除此许可证?"
            onConfirm={() => deleteLicense(record.license_id)}
            okText="确定"
            cancelText="取消"
          >
            <Button type="primary" danger size="small">
              删除
            </Button>
          </Popconfirm>
        </Space>
      ),
    },
  ];

  // Tab 项
  const tabItems = [
    {
      key: 'generate',
      label: '生成许可证',
      children: (
        <div className="license-form">
          <div className="form-group">
            <label htmlFor="machine-code">机器码</label>
            <div className="machine-code-input">
              <Input
                id="machine-code"
                value={machineCode}
                onChange={(e) => setMachineCode(e.target.value)}
                placeholder="输入机器码..."
              />
              <Button onClick={getCurrentMachineId} type="primary" className="get-machine-code-btn">
                获取当前机器码
              </Button>
            </div>
          </div>
          
          <div className="form-group">
            <label htmlFor="customer-name">客户名称</label>
            <Input
              id="customer-name"
              value={customerName}
              onChange={(e) => setCustomerName(e.target.value)}
              placeholder="输入客户名称..."
            />
          </div>
          
          <div className="form-group">
            <label htmlFor="customer-email">客户邮箱</label>
            <Input
              id="customer-email"
              type="email"
              value={customerEmail}
              onChange={(e) => setCustomerEmail(e.target.value)}
              placeholder="输入客户邮箱..."
            />
          </div>
          
          <div className="form-group">
            <label htmlFor="expiry-days">有效期(天)，设为0表示永不过期</label>
            <Input
              id="expiry-days"
              type="number"
              value={expiryDays}
              onChange={(e) => setExpiryDays(e.target.value)}
              placeholder="输入有效期天数..."
            />
          </div>
          
          <div className="form-group">
            <label htmlFor="features">功能权限 (用逗号分隔)</label>
            <Input
              id="features"
              value={features}
              onChange={(e) => setFeatures(e.target.value)}
              placeholder="basic,advanced,export,..."
            />
          </div>
          
          <Button type="primary" onClick={generateLicense} className="generate-btn">生成许可证</Button>
          
          {generatedLicense && (
            <div className="license-result">
              <h3>生成的许可证密钥</h3>
              <Input.TextArea 
                readOnly 
                value={generatedLicense} 
                rows={6}
              />
              <Button type="primary" onClick={() => copyToClipboard(generatedLicense)} className="copy-btn">
                复制许可证
              </Button>
            </div>
          )}
        </div>
      )
    },
    {
      key: 'validate',
      label: '验证许可证',
      children: (
        <div className="validate-form">
          <div className="form-group">
            <label htmlFor="validation-machine-code">机器码</label>
            <div className="machine-code-input">
              <Input
                id="validation-machine-code"
                value={validationMachineCode}
                onChange={(e) => setValidationMachineCode(e.target.value)}
                placeholder="输入机器码..."
              />
              <Button onClick={getCurrentMachineId} type="primary" className="get-machine-code-btn">
                获取当前机器码
              </Button>
            </div>
          </div>
          
          <div className="form-group">
            <label htmlFor="license-key">许可证密钥</label>
            <Input.TextArea
              id="license-key"
              value={licenseKey}
              onChange={(e) => setLicenseKey(e.target.value)}
              placeholder="粘贴许可证密钥..."
              rows={6}
            />
          </div>
          
          <Button type="primary" onClick={validateLicense}>验证许可证</Button>
          
          {validationResult && (
            <div className={`validation-result ${validationResult.is_valid ? 'valid' : 'invalid'}`}>
              <h3>验证结果</h3>
              <p className="validation-message">{validationResult.message}</p>
              
              {validationResult.info && (
                <div className="license-details">
                  <h4>许可证详情:</h4>
                  <ul>
                    <li><strong>许可证 ID:</strong> {validationResult.info.license_id}</li>
                    <li><strong>客户名称:</strong> {validationResult.info.customer_name}</li>
                    <li><strong>客户邮箱:</strong> {validationResult.info.customer_email}</li>
                    <li><strong>发行日期:</strong> {formatDate(validationResult.info.issue_date)}</li>
                    <li><strong>过期日期:</strong> {formatDate(validationResult.info.expiry_date)}</li>
                    <li><strong>功能列表:</strong> {validationResult.info.features.join(', ')}</li>
                    {validationResult.info.machine_code && (
                      <li><strong>绑定机器码:</strong> {validationResult.info.machine_code}</li>
                    )}
                  </ul>
                </div>
              )}
            </div>
          )}
        </div>
      )
    },
    {
      key: 'history',
      label: '历史记录',
      children: (
        <div className="history-view">
          <div className="action-buttons">
            <Button type="primary" onClick={exportPublicKey}>导出当前公钥</Button>
          </div>
          
          <Table 
            dataSource={licenseHistory} 
            columns={columns} 
            rowKey="license_id"
            pagination={{ pageSize: 10 }}
            scroll={{ x: 1000 }}
          />
        </div>
      )
    }
  ];

  return (
    <main className="container">
      <h1>钻井系统许可证管理</h1>
      
      <Tabs 
        activeKey={activeTab}
        onChange={(key) => setActiveTab(key as 'generate' | 'validate' | 'history')}
        items={tabItems}
      />
      
      {/* 公钥模态框 */}
      <Modal
        title="当前RSA公钥"
        open={isPublicKeyModalOpen}
        onCancel={() => setIsPublicKeyModalOpen(false)}
        footer={[
          <Button key="copy" type="primary" onClick={() => copyToClipboard(publicKey)}>
            复制公钥
          </Button>,
          <Button key="close" onClick={() => setIsPublicKeyModalOpen(false)}>
            关闭
          </Button>,
        ]}
        width={800}
      >
        <Input.TextArea 
          readOnly 
          value={publicKey} 
          rows={10}
        />
      </Modal>
    </main>
  );
}

export default App;
