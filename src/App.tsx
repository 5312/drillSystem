import { useState, useEffect } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface LicenseInfo {
  license_id: string;
  customer_name: string;
  customer_email: string;
  issue_date: string;
  expiry_date: string;
  features: string[];
  signature: string;
}

interface LicenseValidationResult {
  is_valid: boolean;
  info?: LicenseInfo;
  message: string;
}

function App() {
  const [activeTab, setActiveTab] = useState<'generate' | 'validate' | 'history'>('generate');
  
  // 生成许可证状态
  const [customerName, setCustomerName] = useState("");
  const [customerEmail, setCustomerEmail] = useState("");
  const [expiryDays, setExpiryDays] = useState("365");
  const [features, setFeatures] = useState("");
  const [generatedLicense, setGeneratedLicense] = useState("");
  
  // 验证许可证状态
  const [licenseKey, setLicenseKey] = useState("");
  const [validationResult, setValidationResult] = useState<LicenseValidationResult | null>(null);
  
  // 历史许可证
  const [licenseHistory, setLicenseHistory] = useState<LicenseInfo[]>([]);
  
  // 公钥
  const [publicKey, setPublicKey] = useState("");
  
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
  
  // 生成许可证
  async function generateLicense() {
    try {
      const featuresList = features.split(',').map(f => f.trim()).filter(f => f);
      const license = await invoke<string>("generate_license_key", { 
        customerName, 
        customerEmail, 
        expiryDays: parseInt(expiryDays), 
        features: featuresList 
      });
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
      const result = await invoke<LicenseValidationResult>("validate_license_key", { 
        licenseKey 
      });
      setValidationResult(result);
    } catch (error) {
      console.error("验证许可证失败:", error);
      setValidationResult({
        is_valid: false,
        message: `错误: ${error}`
      });
    }
  }
  
  // 复制到剪贴板
  const copyToClipboard = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
      alert("已复制到剪贴板");
    } catch (error) {
      console.error("复制到剪贴板失败:", error);
      alert("复制失败: " + error);
    }
  };

  // 格式化日期
  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('zh-CN');
  };

  // 导出公钥
  async function exportPublicKey() {
    try {
      const key = await invoke<string>("export_license_public_key");
      setPublicKey(key);
    } catch (error) {
      console.error("导出公钥失败:", error);
    }
  }

  return (
    <main className="container">
      <h1>钻井系统许可证管理</h1>
      
      <div className="tabs">
        <button 
          className={activeTab === 'generate' ? 'active' : ''} 
          onClick={() => setActiveTab('generate')}
        >
          生成许可证
        </button>
        <button 
          className={activeTab === 'validate' ? 'active' : ''} 
          onClick={() => setActiveTab('validate')}
        >
          验证许可证
        </button>
        <button 
          className={activeTab === 'history' ? 'active' : ''} 
          onClick={() => setActiveTab('history')}
        >
          历史记录
        </button>
      </div>
      
      {activeTab === 'generate' && (
        <div className="license-form">
          <h2>生成许可证</h2>
          <div className="form-group">
            <label htmlFor="customer-name">客户名称</label>
            <input
              id="customer-name"
              value={customerName}
              onChange={(e) => setCustomerName(e.target.value)}
              placeholder="输入客户名称..."
            />
          </div>
          
          <div className="form-group">
            <label htmlFor="customer-email">客户邮箱</label>
            <input
              id="customer-email"
              type="email"
              value={customerEmail}
              onChange={(e) => setCustomerEmail(e.target.value)}
              placeholder="输入客户邮箱..."
            />
          </div>
          
          <div className="form-group">
            <label htmlFor="expiry-days">有效期(天)</label>
            <input
              id="expiry-days"
              type="number"
              value={expiryDays}
              onChange={(e) => setExpiryDays(e.target.value)}
              placeholder="输入有效期天数..."
            />
          </div>
          
          <div className="form-group">
            <label htmlFor="features">功能权限 (用逗号分隔)</label>
            <input
              id="features"
              value={features}
              onChange={(e) => setFeatures(e.target.value)}
              placeholder="basic,advanced,export,..."
            />
          </div>
          
          <button onClick={generateLicense}>生成许可证</button>
          
          {generatedLicense && (
            <div className="license-result">
              <h3>生成的许可证密钥</h3>
              <textarea 
                readOnly 
                value={generatedLicense} 
                rows={6}
              />
              <button onClick={() => copyToClipboard(generatedLicense)}>
                复制许可证
              </button>
            </div>
          )}
        </div>
      )}
      
      {activeTab === 'validate' && (
        <div className="validate-form">
          <h2>验证许可证</h2>
          <div className="form-group">
            <label htmlFor="license-key">许可证密钥</label>
            <textarea
              id="license-key"
              value={licenseKey}
              onChange={(e) => setLicenseKey(e.target.value)}
              placeholder="粘贴许可证密钥..."
              rows={6}
            />
          </div>
          
          <button onClick={validateLicense}>验证许可证</button>
          
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
                  </ul>
                </div>
              )}
            </div>
          )}
        </div>
      )}
      
      {activeTab === 'history' && (
        <div className="history-view">
          <h2>许可证历史记录</h2>
          
          <div className="action-buttons">
            <button onClick={exportPublicKey}>导出公钥</button>
          </div>
          
          {publicKey && (
            <div className="public-key-result">
              <h3>RSA公钥（用于验证许可证）</h3>
              <textarea 
                readOnly 
                value={publicKey} 
                rows={6}
              />
              <button onClick={() => copyToClipboard(publicKey)}>
                复制公钥
              </button>
            </div>
          )}

          {licenseHistory.length === 0 ? (
            <p>暂无许可证记录</p>
          ) : (
            <div className="license-list">
              {licenseHistory.map(license => (
                <div key={license.license_id} className="license-card">
                  <h3>{license.customer_name}</h3>
                  <ul>
                    <li><strong>邮箱:</strong> {license.customer_email}</li>
                    <li><strong>发行日期:</strong> {formatDate(license.issue_date)}</li>
                    <li><strong>过期日期:</strong> {formatDate(license.expiry_date)}</li>
                    <li><strong>功能:</strong> {license.features.join(', ')}</li>
                  </ul>
                </div>
              ))}
            </div>
          )}
        </div>
      )}
    </main>
  );
}

export default App;
