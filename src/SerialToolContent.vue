<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
  NLayout,
  NLayoutHeader,
  NLayoutContent,
  NCard,
  NSpace,
  NSelect,
  NButton,
  NInput,
  NSwitch,
  NScrollbar,
  NStatistic,
  NGrid,
  NGi,
  useMessage,
} from "naive-ui";
import {
  PlayCircle,
  StopCircle,
  RefreshCircle,
  TrashBin,
  Download,
} from "@vicons/ionicons5";

// 串口信息接口
interface SerialPortInfo {
  port_name: string;
  port_type: string;
}

// 使用 message API
const message = useMessage();

// 状态管理
const serialPorts = ref<SerialPortInfo[]>([]);
const selectedPort = ref<string | null>(null);
const isConnected = ref(false);
const baudRate = ref(115200);
const dataBits = ref(8);
const stopBits = ref(1);
const parity = ref("None");
const isHexMode = ref(false);
const showTimestamp = ref(true);
const autoScroll = ref(true);
const sendData = ref("");
const receiveData = ref("");
const sentBytes = ref(0);
const receivedBytes = ref(0);

let readInterval: number | null = null;

// 波特率选项
const baudRateOptions = [
  { label: "9600", value: 9600 },
  { label: "19200", value: 19200 },
  { label: "38400", value: 38400 },
  { label: "57600", value: 57600 },
  { label: "115200", value: 115200 },
  { label: "230400", value: 230400 },
  { label: "460800", value: 460800 },
];

// 数据位选项
const dataBitsOptions = [
  { label: "5", value: 5 },
  { label: "6", value: 6 },
  { label: "7", value: 7 },
  { label: "8", value: 8 },
];

// 停止位选项
const stopBitsOptions = [
  { label: "1", value: 1 },
  { label: "2", value: 2 },
];

// 校验位选项
const parityOptions = [
  { label: "None", value: "None" },
  { label: "Odd", value: "Odd" },
  { label: "Even", value: "Even" },
];

// 刷新串口列表
async function refreshPorts() {
  try {
    serialPorts.value = await invoke<SerialPortInfo[]>("list_serial_ports");
    message.success(`找到 ${serialPorts.value.length} 个串口`);
  } catch (error) {
    message.error(`刷新串口失败: ${error}`);
  }
}

// 连接串口
async function connectPort() {
  if (!selectedPort.value) {
    message.warning("请选择串口");
    return;
  }

  try {
    const config = {
      port_name: selectedPort.value,
      baud_rate: baudRate.value,
      data_bits: dataBits.value,
      stop_bits: stopBits.value,
      parity: parity.value,
    };

    await invoke("open_serial_port", { config });
    isConnected.value = true;
    message.success(`串口 ${selectedPort.value} 已连接`);

    // 启动数据读取定时器
    startReading();
  } catch (error) {
    message.error(`连接失败: ${error}`);
  }
}

// 断开串口
async function disconnectPort() {
  if (!selectedPort.value) return;

  try {
    // 停止读取
    stopReading();

    await invoke("close_serial_port", { port_name: selectedPort.value });
    isConnected.value = false;
    message.success("串口已断开");
  } catch (error) {
    message.error(`断开失败: ${error}`);
  }
}

// 发送数据
async function sendSerialData() {
  if (!selectedPort.value || !isConnected.value) {
    message.warning("请先连接串口");
    return;
  }

  if (!sendData.value.trim()) {
    message.warning("请输入要发送的数据");
    return;
  }

  try {
    const result = await invoke<string>("write_serial_data", {
      port_name: selectedPort.value,
      data: sendData.value,
      is_hex: isHexMode.value,
    });

    // 更新发送字节数
    const bytes = parseInt(result.match(/\d+/)?.[0] || "0");
    sentBytes.value += bytes;

    // 在接收区显示发送的数据
    addReceiveData(`[发送] ${sendData.value}`, "send");
    message.success(result);

    // 清空发送框
    sendData.value = "";
  } catch (error) {
    message.error(`发送失败: ${error}`);
  }
}

// 启动数据读取
function startReading() {
  readInterval = window.setInterval(async () => {
    if (!selectedPort.value || !isConnected.value) return;

    try {
      const data = await invoke<number[]>("read_serial_data", {
        port_name: selectedPort.value,
        timeout_ms: 50,
      });

      if (data.length > 0) {
        receivedBytes.value += data.length;
        const text = isHexMode.value
          ? data.map((b) => b.toString(16).padStart(2, "0").toUpperCase()).join(" ")
          : new TextDecoder().decode(new Uint8Array(data));

        addReceiveData(text, "receive");
      }
    } catch (error) {
      console.error("读取数据失败:", error);
    }
  }, 100);
}

// 停止数据读取
function stopReading() {
  if (readInterval !== null) {
    clearInterval(readInterval);
    readInterval = null;
  }
}

// 添加接收数据
function addReceiveData(text: string, type: "send" | "receive") {
  const timestamp = showTimestamp.value
    ? `[${new Date().toLocaleTimeString()}] `
    : "";
  const prefix = type === "send" ? "[TX] " : "[RX] ";
  receiveData.value += timestamp + prefix + text + "\n";

  // 自动滚动
  if (autoScroll.value) {
    setTimeout(() => {
      const container = document.querySelector(".receive-container");
      if (container) {
        container.scrollTop = container.scrollHeight;
      }
    }, 10);
  }
}

// 清空接收区
function clearReceiveData() {
  receiveData.value = "";
  sentBytes.value = 0;
  receivedBytes.value = 0;
  message.info("已清空");
}

// 保存日志
function saveLog() {
  const blob = new Blob([receiveData.value], { type: "text/plain" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `serial-log-${Date.now()}.txt`;
  a.click();
  URL.revokeObjectURL(url);
  message.success("日志已保存");
}

// 生命周期
onMounted(() => {
  refreshPorts();
});

onUnmounted(() => {
  stopReading();
  if (isConnected.value && selectedPort.value) {
    invoke("close_serial_port", { port_name: selectedPort.value });
  }
});
</script>

<template>
  <n-layout class="app-layout">
    <n-layout-header class="header">
      <div class="header-content">
        <h1 class="title">
          <span class="icon">⚡</span>
          串口调试助手
        </h1>
      </div>
    </n-layout-header>

    <n-layout-content class="main-content">
      <div class="content-wrapper">
        <!-- 左侧控制面板 -->
        <n-card class="glass-card control-panel" title="连接配置">
          <n-space vertical :size="16">
            <!-- 串口选择 -->
            <div>
              <label class="label">串口选择</label>
              <n-space>
                <n-select
                  v-model:value="selectedPort"
                  :options="serialPorts.map(p => ({ label: `${p.port_name} (${p.port_type})`, value: p.port_name }))"
                  placeholder="选择串口"
                  :disabled="isConnected"
                  style="flex: 1; min-width: 200px"
                />
                <n-button
                  @click="refreshPorts"
                  :disabled="isConnected"
                  secondary
                >
                  <template #icon>
                    <RefreshCircle />
                  </template>
                </n-button>
              </n-space>
            </div>

            <!-- 波特率 -->
            <div>
              <label class="label">波特率</label>
              <n-select
                v-model:value="baudRate"
                :options="baudRateOptions"
                :disabled="isConnected"
              />
            </div>

            <!-- 数据位、停止位、校验位 -->
            <n-grid :cols="3" :x-gap="12">
              <n-gi>
                <label class="label">数据位</label>
                <n-select
                  v-model:value="dataBits"
                  :options="dataBitsOptions"
                  :disabled="isConnected"
                />
              </n-gi>
              <n-gi>
                <label class="label">停止位</label>
                <n-select
                  v-model:value="stopBits"
                  :options="stopBitsOptions"
                  :disabled="isConnected"
                />
              </n-gi>
              <n-gi>
                <label class="label">校验位</label>
                <n-select
                  v-model:value="parity"
                  :options="parityOptions"
                  :disabled="isConnected"
                />
              </n-gi>
            </n-grid>

            <!-- 连接按钮 -->
            <n-button
              v-if="!isConnected"
              @click="connectPort"
              type="success"
              size="large"
              block
            >
              <template #icon>
                <PlayCircle />
              </template>
              连接串口
            </n-button>
            <n-button
              v-else
              @click="disconnectPort"
              type="error"
              size="large"
              block
            >
              <template #icon>
                <StopCircle />
              </template>
              断开连接
            </n-button>

            <!-- 统计信息 -->
            <n-grid :cols="2" :x-gap="12">
              <n-gi>
                <n-statistic label="发送字节" :value="sentBytes" />
              </n-gi>
              <n-gi>
                <n-statistic label="接收字节" :value="receivedBytes" />
              </n-gi>
            </n-grid>

            <!-- 显示选项 -->
            <div class="options">
              <div class="option-item">
                <span>十六进制模式</span>
                <n-switch v-model:value="isHexMode" />
              </div>
              <div class="option-item">
                <span>显示时间戳</span>
                <n-switch v-model:value="showTimestamp" />
              </div>
              <div class="option-item">
                <span>自动滚动</span>
                <n-switch v-model:value="autoScroll" />
              </div>
            </div>
          </n-space>
        </n-card>

        <!-- 右侧主区域 -->
        <div class="main-area">
          <!-- 接收区 -->
          <n-card class="glass-card receive-card" title="接收区">
            <template #header-extra>
              <n-space>
                <n-button @click="saveLog" secondary size="small">
                  <template #icon>
                    <Download />
                  </template>
                  保存
                </n-button>
                <n-button @click="clearReceiveData" secondary size="small">
                  <template #icon>
                    <TrashBin />
                  </template>
                  清空
                </n-button>
              </n-space>
            </template>
            <n-scrollbar class="receive-container">
              <pre class="receive-text">{{ receiveData || '等待接收数据...' }}</pre>
            </n-scrollbar>
          </n-card>

          <!-- 发送区 -->
          <n-card class="glass-card send-card" title="发送区">
            <n-space vertical>
              <n-input
                v-model:value="sendData"
                type="textarea"
                :placeholder="isHexMode ? '输入十六进制数据，例：01 02 03' : '输入要发送的数据'"
                :rows="3"
                @keydown.enter.ctrl="sendSerialData"
              />
              <n-button
                @click="sendSerialData"
                type="primary"
                :disabled="!isConnected"
                block
              >
                发送 (Ctrl+Enter)
              </n-button>
            </n-space>
          </n-card>
        </div>
      </div>
    </n-layout-content>
  </n-layout>
</template>

<style scoped>
/* 全局布局 */
.app-layout {
  height: 100vh;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  position: relative;
  overflow: hidden;
}

.app-layout::before {
  content: '';
  position: absolute;
  top: -50%;
  left: -50%;
  width: 200%;
  height: 200%;
  background: radial-gradient(circle, rgba(255,255,255,0.1) 1px, transparent 1px);
  background-size: 50px 50px;
  animation: backgroundMove 20s linear infinite;
}

@keyframes backgroundMove {
  0% { transform: translate(0, 0); }
  100% { transform: translate(50px, 50px); }
}

/* 头部 */
.header {
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(10px);
  border-bottom: 1px solid rgba(255, 255, 255, 0.2);
  padding: 0 !important;
  height: 64px;
  display: flex;
  align-items: center;
}

.header-content {
  width: 100%;
  padding: 0 24px;
}

.title {
  margin: 0;
  font-size: 24px;
  font-weight: 600;
  color: white;
  display: flex;
  align-items: center;
  gap: 12px;
}

.title .icon {
  font-size: 28px;
  animation: pulse 2s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.6; }
}

/* 主内容区 */
.main-content {
  padding: 24px;
  height: calc(100vh - 64px);
  position: relative;
  z-index: 1;
}

.content-wrapper {
  display: grid;
  grid-template-columns: 320px 1fr;
  gap: 24px;
  height: 100%;
}

/* 玻璃拟态卡片 */
.glass-card {
  background: rgba(255, 255, 255, 0.1) !important;
  backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.2);
  border-radius: 16px;
  box-shadow: 0 8px 32px 0 rgba(31, 38, 135, 0.37);
}

.glass-card :deep(.n-card-header) {
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  color: white;
  font-weight: 600;
}

.glass-card :deep(.n-card__content) {
  color: rgba(255, 255, 255, 0.9);
}

/* 控制面板 */
.control-panel {
  height: 100%;
  overflow-y: auto;
}

.label {
  display: block;
  margin-bottom: 8px;
  color: rgba(255, 255, 255, 0.9);
  font-size: 14px;
  font-weight: 500;
}

.options {
  background: rgba(0, 0, 0, 0.2);
  border-radius: 12px;
  padding: 16px;
}

.option-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 0;
  color: rgba(255, 255, 255, 0.9);
  font-size: 14px;
}

.option-item:not(:last-child) {
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

/* 主区域 */
.main-area {
  display: flex;
  flex-direction: column;
  gap: 24px;
  height: 100%;
}

.receive-card {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.receive-card :deep(.n-card__content) {
  flex: 1;
  min-height: 0;
  padding: 0 !important;
}

.receive-container {
  height: 100%;
  background: rgba(0, 0, 0, 0.3);
  border-radius: 8px;
  padding: 16px;
}

.receive-text {
  margin: 0;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 13px;
  line-height: 1.6;
  color: #00ff00;
  white-space: pre-wrap;
  word-break: break-all;
}

.send-card {
  flex-shrink: 0;
}

/* 自定义滚动条 */
:deep(.n-scrollbar-rail) {
  background: rgba(255, 255, 255, 0.1);
}

:deep(.n-scrollbar-rail__scrollbar) {
  background: rgba(255, 255, 255, 0.3);
}

/* 输入框样式 */
:deep(.n-input),
:deep(.n-select) {
  background: rgba(0, 0, 0, 0.2) !important;
  border-color: rgba(255, 255, 255, 0.2) !important;
  color: white !important;
}

:deep(.n-input__textarea-el),
:deep(.n-input__input-el) {
  color: white !important;
}

:deep(.n-base-selection),
:deep(.n-base-selection-label) {
  background: rgba(0, 0, 0, 0.2) !important;
  color: white !important;
}

/* 按钮样式 */
:deep(.n-button) {
  font-weight: 500;
}

/* 统计信息 */
:deep(.n-statistic) {
  background: rgba(0, 0, 0, 0.2);
  border-radius: 8px;
  padding: 12px;
}

:deep(.n-statistic-value__content) {
  color: #00ff88 !important;
  font-weight: 600;
}

:deep(.n-statistic .n-statistic-value__prefix),
:deep(.n-statistic .n-statistic-value__suffix) {
  color: #00ff88 !important;
}
</style>
