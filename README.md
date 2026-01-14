# 串口调试助手

一款基于 Tauri + Vue 3 开发的桌面端串口调试工具，支持真实串口和虚拟串口模拟。

## ✨ 特性

- 🔌 支持真实串口通信（COM1-COM256）
- 🎮 内置 3 种虚拟串口模式（Echo、Reply、Random）
- 📊 数据支持文本和 HEX 两种格式
- 🎨 玻璃拟态风格界面
- 💾 支持日志保存和数据统计
- 🪟 完美支持 Windows 系统

## 技术栈

- **前端**: Vue 3 + TypeScript + Vite + Naive UI
- **后端**: Rust + Tauri 2
- **串口**: serialport 4.5

## 开发环境

```bash
# 安装依赖
npm install

# 开发模式
npm run tauri dev

# 构建应用
npm run tauri build
```

## 推荐 IDE

- [VS Code](https://code.visualstudio.com/) + [Vue - Official](https://marketplace.visualstudio.com/items?itemName=Vue.volar) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
