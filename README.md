# 钻井系统许可证管理

基于Tauri构建的钻井系统许可证管理工具，支持自动更新功能。

## 功能特性

- 许可证管理：创建、验证和管理许可证
- 自动更新：通过GitHub自动更新应用程序
- 跨平台：支持Windows、macOS和Linux

## 开发环境设置

### 前置条件

1. [Node.js](https://nodejs.org/) (推荐v18+)
2. [Rust](https://www.rust-lang.org/tools/install)
3. [pnpm](https://pnpm.io/installation)
4. [Tauri开发环境](https://tauri.app/v2/guides/prerequisites/)

### 安装依赖

```bash
# 安装依赖
pnpm install
```

### 开发运行

```bash
# 开发模式运行
pnpm tauri dev
```

### 构建应用

```bash
# 构建生产版本
pnpm tauri build
```

## 自动更新配置

本应用使用GitHub Actions进行自动构建和发布，支持自动更新功能。

### 生成更新密钥

```bash
# 生成更新密钥
pnpm generate-update-key
```

### GitHub Actions配置

在GitHub仓库设置中添加以下Secrets：

- `TAURI_SIGNING_PRIVATE_KEY`: 私钥内容
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`: 私钥密码

## 发布新版本

1. 更新`package.json`中的版本号
2. 提交代码并推送
3. 创建新的tag（格式为`v*.*.*`）并推送
4. GitHub Actions会自动构建和发布新版本
5. ARM64 构建只支持公共库
## 许可证

MIT
