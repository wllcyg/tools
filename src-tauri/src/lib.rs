use serde::{Deserialize, Serialize};
use serialport::{SerialPort, SerialPortType};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use once_cell::sync::Lazy;
use std::collections::HashMap;

// 全局串口连接管理器
static SERIAL_PORTS: Lazy<Arc<Mutex<HashMap<String, Box<dyn SerialPort>>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

// 虚拟串口数据缓冲（简化实现，移除未使用的 channel）
static VIRTUAL_BUFFERS: Lazy<Arc<Mutex<HashMap<String, Vec<u8>>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

// 串口信息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerialPortInfo {
    port_name: String,
    port_type: String,
}

// 串口配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerialConfig {
    port_name: String,
    baud_rate: u32,
    data_bits: u8,
    stop_bits: u8,
    parity: String,
}

// 列出所有可用串口
#[tauri::command]
fn list_serial_ports() -> Result<Vec<SerialPortInfo>, String> {
    let mut port_list: Vec<SerialPortInfo> = vec![];
    
    // 添加真实串口
    if let Ok(ports) = serialport::available_ports() {
        port_list.extend(
            ports.iter().map(|p| {
                let port_type = match &p.port_type {
                    SerialPortType::UsbPort(info) => {
                        // 显示 USB 设备详细信息
                        if let Some(product) = &info.product {
                            format!("USB: {}", product)
                        } else if let Some(manufacturer) = &info.manufacturer {
                            format!("USB: {}", manufacturer)
                        } else {
                            "USB Device".to_string()
                        }
                    }
                    SerialPortType::PciPort => "PCI Port".to_string(),
                    SerialPortType::BluetoothPort => "Bluetooth".to_string(),
                    SerialPortType::Unknown => "Unknown".to_string(),
                };
                SerialPortInfo {
                    port_name: p.port_name.clone(),
                    port_type,
                }
            })
        );
    }
    
    // 添加虚拟串口
    port_list.push(SerialPortInfo {
        port_name: "VIRTUAL-COM1".to_string(),
        port_type: "Virtual Port (Echo)".to_string(),
    });
    port_list.push(SerialPortInfo {
        port_name: "VIRTUAL-COM2".to_string(),
        port_type: "Virtual Port (Reply)".to_string(),
    });
    port_list.push(SerialPortInfo {
        port_name: "VIRTUAL-COM3".to_string(),
        port_type: "Virtual Port (Random)".to_string(),
    });
    
    Ok(port_list)
}

// 打开串口
#[tauri::command]
fn open_serial_port(config: SerialConfig) -> Result<String, String> {
    // 检查是否是虚拟串口
    if config.port_name.starts_with("VIRTUAL-") {
        // 初始化虚拟串口缓冲区
        let mut buffers = VIRTUAL_BUFFERS.lock()
            .expect("Failed to lock VIRTUAL_BUFFERS mutex");
        buffers.insert(config.port_name.clone(), Vec::new());
        
        return Ok(format!("Virtual port {} opened successfully", config.port_name));
    }
    
    // 真实串口逻辑 - Windows 兼容性优化
    // 解析校验位
    let parity = match config.parity.as_str() {
        "None" => serialport::Parity::None,
        "Odd" => serialport::Parity::Odd,
        "Even" => serialport::Parity::Even,
        _ => serialport::Parity::None,
    };

    // 解析停止位
    let stop_bits = match config.stop_bits {
        1 => serialport::StopBits::One,
        2 => serialport::StopBits::Two,
        _ => serialport::StopBits::One,
    };

    // 解析数据位
    let data_bits = match config.data_bits {
        5 => serialport::DataBits::Five,
        6 => serialport::DataBits::Six,
        7 => serialport::DataBits::Seven,
        8 => serialport::DataBits::Eight,
        _ => serialport::DataBits::Eight,
    };

    // Windows 串口路径格式化（处理 COM10+ 的情况）
    let port_path = normalize_port_path(&config.port_name);

    // 打开串口
    let port = serialport::new(&port_path, config.baud_rate)
        .data_bits(data_bits)
        .stop_bits(stop_bits)
        .parity(parity)
        .timeout(Duration::from_millis(100))
        .open()
        .map_err(|e| format!("Failed to open port: {}", e))?;

    // 保存到全局管理器
    let mut ports = SERIAL_PORTS.lock()
        .expect("Failed to lock SERIAL_PORTS mutex");
    ports.insert(config.port_name.clone(), port);

    Ok(format!("Port {} opened successfully", config.port_name))
}

// 关闭串口
#[tauri::command]
fn close_serial_port(port_name: String) -> Result<String, String> {
    // 检查是否是虚拟串口
    if port_name.starts_with("VIRTUAL-") {
        let mut buffers = VIRTUAL_BUFFERS.lock()
            .expect("Failed to lock VIRTUAL_BUFFERS mutex");
        buffers.remove(&port_name);
        
        return Ok(format!("Virtual port {} closed successfully", port_name));
    }
    
    // 真实串口逻辑
    let mut ports = SERIAL_PORTS.lock()
        .expect("Failed to lock SERIAL_PORTS mutex");
    
    if ports.remove(&port_name).is_some() {
        Ok(format!("Port {} closed successfully", port_name))
    } else {
        Err(format!("Port {} not found", port_name))
    }
}

// 发送数据
#[tauri::command]
fn write_serial_data(port_name: String, data: String, is_hex: bool) -> Result<String, String> {
    let bytes_to_send: Vec<u8> = if is_hex {
        hex_string_to_bytes(&data)
            .map_err(|e| format!("Invalid hex string: {}", e))?
    } else {
        data.as_bytes().to_vec()
    };
    
    // 虚拟串口逻辑
    if port_name.starts_with("VIRTUAL-") {
        let mut buffers = VIRTUAL_BUFFERS.lock()
            .expect("Failed to lock VIRTUAL_BUFFERS mutex");
        
        if let Some(buffer) = buffers.get_mut(&port_name) {
            // 根据不同的虚拟串口类型执行不同操作
            match port_name.as_str() {
                "VIRTUAL-COM1" => {
                    // Echo 模式：原样返回
                    buffer.extend_from_slice(&bytes_to_send);
                }
                "VIRTUAL-COM2" => {
                    // Reply 模式：返回固定回复
                    let reply = format!("Received: {}", data);
                    buffer.extend_from_slice(reply.as_bytes());
                }
                "VIRTUAL-COM3" => {
                    // Random 模式：返回随机数据
                    use std::time::{SystemTime, UNIX_EPOCH};
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis();
                    let random_data = format!("Random-{}", timestamp % 10000);
                    buffer.extend_from_slice(random_data.as_bytes());
                }
                _ => {}
            }
            
            return Ok(format!("Sent {} bytes", bytes_to_send.len()));
        } else {
            return Err(format!("Virtual port {} not found", port_name));
        }
    }
    
    // 真实串口逻辑
    let mut ports = SERIAL_PORTS.lock()
        .expect("Failed to lock SERIAL_PORTS mutex");
    
    let port = ports
        .get_mut(&port_name)
        .ok_or_else(|| format!("Port {} not found", port_name))?;

    port.write_all(&bytes_to_send)
        .map_err(|e| format!("Failed to write data: {}", e))?;

    Ok(format!("Sent {} bytes", bytes_to_send.len()))
}

// 读取数据
#[tauri::command]
fn read_serial_data(port_name: String, timeout_ms: u64) -> Result<Vec<u8>, String> {
    // 虚拟串口逻辑
    if port_name.starts_with("VIRTUAL-") {
        let mut buffers = VIRTUAL_BUFFERS.lock()
            .expect("Failed to lock VIRTUAL_BUFFERS mutex");
        
        if let Some(buffer) = buffers.get_mut(&port_name) {
            if buffer.is_empty() {
                return Ok(vec![]);
            }
            
            // 读取所有缓冲数据
            let data = buffer.clone();
            buffer.clear();
            return Ok(data);
        } else {
            return Err(format!("Virtual port {} not found", port_name));
        }
    }
    
    // 真实串口逻辑
    let mut ports = SERIAL_PORTS.lock()
        .expect("Failed to lock SERIAL_PORTS mutex");
    
    let port = ports
        .get_mut(&port_name)
        .ok_or_else(|| format!("Port {} not found", port_name))?;

    // 设置超时
    port.set_timeout(Duration::from_millis(timeout_ms))
        .map_err(|e| format!("Failed to set timeout: {}", e))?;

    let mut buffer: Vec<u8> = vec![0; 1024];
    match port.read(&mut buffer) {
        Ok(n) => {
            buffer.truncate(n);
            Ok(buffer)
        }
        Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
            Ok(vec![])
        }
        Err(e) => Err(format!("Failed to read data: {}", e)),
    }
}

// HEX字符串转字节数组
fn hex_string_to_bytes(hex: &str) -> Result<Vec<u8>, String> {
    let hex_clean: String = hex.chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    
    if hex_clean.len() % 2 != 0 {
        return Err("Hex string length must be even".to_string());
    }

    (0..hex_clean.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex_clean[i..i + 2], 16)
                .map_err(|e| format!("Invalid hex: {}", e))
        })
        .collect()
}

// Windows 串口路径规范化函数
// Windows 上 COM10 及以上需要使用 \\.\COMx 格式
fn normalize_port_path(port_name: &str) -> String {
    if cfg!(target_os = "windows") {
        // 检查是否是 COM 端口
        if let Some(stripped) = port_name.strip_prefix("COM") {
            if let Ok(num) = stripped.parse::<u32>() {
                // COM10 及以上使用完整路径格式
                if num >= 10 {
                    return format!("\\\\.\\{}", port_name);
                }
            }
        }
    }
    port_name.to_string()
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            list_serial_ports,
            open_serial_port,
            close_serial_port,
            write_serial_data,
            read_serial_data
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
