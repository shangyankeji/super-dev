---
id: edge-iot-complete
title: 边缘计算与IoT完整指南
domain: edge-iot
category: 01-standards
difficulty: intermediate
tags: [complete, edge, edge-iot, iot, 参考资料, 学习路径, 最佳实践, 核心概念]
quality_score: 70
last_updated: 2026-06-15
---
# 边缘计算与IoT完整指南

## 概述
边缘计算将计算能力下沉到网络边缘(设备端),减少延迟和带宽消耗。IoT(物联网)连接数十亿设备,从智能家居到工业4.0。本指南覆盖边缘架构、IoT协议、数据处理和最佳实践。

## 核心概念

### 1. 边缘计算架构

**三层架构**:
```
Cloud (云端)
  ├── 数据中心、AI训练、长期存储
  |
Edge (边缘)
  ├── 边缘服务器、5G基站、CDN节点
  |
Device (设备端)
  ├── 传感器、摄像头、智能设备
```

**实现**:
```python
# 边缘节点数据处理
import numpy as np
from typing import Dict

class EdgeNode:
    def __init__(self, node_id: str, capacity: int):
        self.node_id = node_id
        self.capacity = capacity  # 处理能力(每秒样本数)
        self.buffer = []
        self.model = None
    
    def load_model(self, model_path: str):
        """加载轻量级ML模型"""
        import tensorflow as tf
        self.model = tf.lite.Interpreter(model_path=model_path)
        self.model.allocate_tensors()
    
    def process_stream(self, data: np.ndarray) -> Dict:
        """实时处理数据流"""
        # 缓冲数据
        self.buffer.append(data)
        
        # 批量处理
        if len(self.buffer) >= self.capacity:
            batch = np.array(self.buffer)
            
            # 本地推理
            results = self.inference(batch)
            
            # 清空缓冲
            self.buffer = []
            
            return results
        
        return None
    
    def inference(self, batch: np.ndarray) -> Dict:
        """边缘推理"""
        input_details = self.model.get_input_details()
        output_details = self.model.get_output_details()
        
        self.model.set_tensor(input_details[0]['index'], batch)
        self.model.invoke()
        
        output = self.model.get_tensor(output_details[0]['index'])
        
        # 只上传异常数据到云端
        anomalies = self.filter_anomalies(output)
        return anomalies
    
    def filter_anomalies(self, predictions: np.ndarray) -> Dict:
        """过滤异常,减少上传"""
        threshold = 0.9
        anomalies = []
        
        for idx, pred in enumerate(predictions):
            if pred > threshold:
                anomalies.append({
                    'index': idx,
                    'confidence': float(pred),
                    'timestamp': time.time()
                })
        
        return {
            'node_id': self.node_id,
            'anomalies': anomalies,
            'total_processed': len(predictions)
        }

# 使用示例
edge = EdgeNode('factory-line-1', capacity=32)
edge.load_model('anomaly_detection.tflite')

# 模拟数据流
for i in range(100):
    sensor_data = np.random.randn(32, 10)  # 32个样本,10个特征
    result = edge.process_stream(sensor_data)
    
    if result and result['anomalies']:
        # 上传到云端
        upload_to_cloud(result)
```

### 2. IoT协议

**MQTT**:
```python
import paho.mqtt.client as mqtt
import json

class IoTSensor:
    def __init__(self, broker: str, port: int, topic: str):
        self.client = mqtt.Client()
        self.broker = broker
        self.port = port
        self.topic = topic
        
        # 设置回调
        self.client.on_connect = self.on_connect
        self.client.on_message = self.on_message
    
    def on_connect(self, client, userdata, flags, rc):
        print(f"Connected with result code {rc}")
        client.subscribe(self.topic)
    
    def on_message(self, client, userdata, msg):
        payload = json.loads(msg.payload.decode())
        self.process_message(payload)
    
    def process_message(self, payload: dict):
        """处理传感器数据"""
        temperature = payload['temperature']
        humidity = payload['humidity']
        
        # 边缘计算: 异常检测
        if temperature > 50 or humidity > 90:
            self.alert_anomaly(payload)
        else:
            self.store_locally(payload)
    
    def alert_anomaly(self, data: dict):
        """异常告警"""
        alert = {
            'type': 'anomaly',
            'sensor_id': data['sensor_id'],
            'values': {
                'temperature': data['temperature'],
                'humidity': data['humidity']
            },
            'timestamp': time.time()
        }
        self.client.publish('alerts/anomaly', json.dumps(alert))
    
    def store_locally(self, data: dict):
        """本地存储(减少云端带宽)"""
        with open('sensor_data.log', 'a') as f:
            f.write(json.dumps(data) + '\n')
    
    def connect(self):
        self.client.connect(self.broker, self.port, 60)
        self.client.loop_start()

# 使用
sensor = IoTSensor('mqtt.broker.com', 1883, 'sensors/temperature')
sensor.connect()

# 模拟传感器发送数据
import time
while True:
    data = {
        'sensor_id': 'temp-001',
        'temperature': np.random.uniform(20, 55),
        'humidity': np.random.uniform(30, 95),
        'timestamp': time.time()
    }
    sensor.client.publish('sensors/temperature', json.dumps(data))
    time.sleep(5)
```

**CoAP**:
```python
from coapthon.client.helperclient import HelperClient
from coapthon.resources.resource import Resource

class SensorResource(Resource):
    def __init__(self, name="sensor"):
        super(SensorResource, self).__init__(name)
        self.payload = "Temperature: 25°C"
    
    def render_GET(self, request):
        return self
    
    def render_POST(self, request):
        # 接收传感器数据
        data = request.payload
        self.process_data(data)
        return self
    
    def process_data(self, data):
        """处理传感器POST数据"""
        print(f"Received: {data}")

# CoAP服务器
from coapthon.server.coap import CoAP

coap_server = CoAP("0.0.0.0", 5683)
coap_server.add_resource('sensor/', SensorResource())

try:
    coap_server.listen(10)
except KeyboardInterrupt:
    coap_server.close()
```

### 3. 时序数据处理

**流处理**:
```python
from collections import deque
import numpy as np

class TimeSeriesProcessor:
    def __init__(self, window_size: int = 100):
        self.window = deque(maxlen=window_size)
        self.anomaly_threshold = 3.0  # 3倍标准差
    
    def add_point(self, value: float) -> dict:
        """添加新数据点并检测异常"""
        self.window.append(value)
        
        if len(self.window) < 10:
            return {'status': 'insufficient_data'}
        
        # 计算统计量
        mean = np.mean(self.window)
        std = np.std(self.window)
        
        # Z-score异常检测
        z_score = (value - mean) / std if std > 0 else 0
        
        result = {
            'value': value,
            'mean': mean,
            'std': std,
            'z_score': z_score,
            'is_anomaly': abs(z_score) > self.anomaly_threshold
        }
        
        return result
    
    def predict_next(self) -> float:
        """简单移动平均预测"""
        if len(self.window) == 0:
            return 0.0
        
        # 加权移动平均
        weights = np.exp(np.linspace(-1, 0, len(self.window)))
        weights /= weights.sum()
        
        prediction = np.dot(weights, self.window)
        return prediction

# 使用
processor = TimeSeriesProcessor(window_size=50)

for i in range(100):
    # 模拟传感器数据
    value = np.random.normal(25, 2)  # 温度数据
    
    # 随机注入异常
    if i == 50:
        value = 50  # 异常高温
    
    result = processor.add_point(value)
    
    if result['is_anomaly']:
        print(f"⚠️ 检测到异常! 值={value}, Z-score={result['z_score']:.2f}")
    
    # 预测下一个值
    prediction = processor.predict_next()
```

### 4. 设备管理

**OTA更新**:
```python
import hashlib
import json

class OTAUpdater:
    def __init__(self, device_id: str):
        self.device_id = device_id
        self.firmware_version = "1.0.0"
    
    def check_update(self) -> dict:
        """检查固件更新"""
        # 从服务器获取最新版本
        latest = self.fetch_latest_version()
        
        if self.compare_versions(latest['version'], self.firmware_version):
            return {
                'update_available': True,
                'version': latest['version'],
                'size': latest['size'],
                'checksum': latest['checksum']
            }
        
        return {'update_available': False}
    
    def download_firmware(self, version: str) -> bytes:
        """下载固件"""
        import requests
        url = f"https://firmware.example.com/{version}.bin"
        
        response = requests.get(url)
        return response.content
    
    def verify_firmware(self, firmware: bytes, expected_checksum: str) -> bool:
        """验证固件完整性"""
        actual = hashlib.sha256(firmware).hexdigest()
        return actual == expected_checksum
    
    def apply_update(self, firmware: bytes):
        """应用固件更新"""
        # 1. 验证
        if not self.verify_firmware(firmware, expected_checksum):
            raise ValueError("Firmware verification failed")
        
        # 2. 写入临时分区
        self.write_to_temp_partition(firmware)
        
        # 3. 验证新固件可启动
        if not self.verify_bootable():
            self.rollback()
            raise RuntimeError("Firmware not bootable")
        
        # 4. 切换启动分区
        self.switch_boot_partition()
        
        # 5. 重启设备
        self.reboot()

# 使用
updater = OTAUpdater('device-001')
update_info = updater.check_update()

if update_info['update_available']:
    print(f"发现新版本: {update_info['version']}")
    
    firmware = updater.download_firmware(update_info['version'])
    
    try:
        updater.apply_update(firmware)
        print("✅ 更新成功")
    except Exception as e:
        print(f"❌ 更新失败: {e}")
```

## 最佳实践

### ✅ DO

1. **边缘过滤数据**
```python
# ✅ 只上传有价值数据
if is_anomaly(data):
    upload_to_cloud(data)
else:
    store_locally(data)
```

2. **使用轻量级模型**
```python
# ✅ TensorFlow Lite
converter = tf.lite.TFLiteConverter.from_keras_model(model)
tflite_model = converter.convert()

# 量化减小模型大小
converter.optimizations = [tf.lite.Optimize.DEFAULT]
```

3. **断网容错**
```python
# ✅ 本地缓存+稍后同步
class ResilientSensor:
    def __init__(self):
        self.cache = []
    
    def send_data(self, data):
        try:
            upload_to_cloud(data)
        except ConnectionError:
            self.cache.append(data)
            self.retry_later()
```

### ❌ DON'T

1. **不要实时上传所有数据**
```python
# ❌ 带宽浪费
for sample in sensor_stream:
    upload_to_cloud(sample)

# ✅ 边缘聚合
batch = aggregate(samples, batch_size=100)
upload_to_cloud(batch)
```

2. **不要忽视安全**
```python
# ❌ 明文通信
client.publish('sensors', json.dumps(data))

# ✅ 加密通信
encrypted = encrypt(data, key)
client.publish('sensors', encrypted)
```

## 学习路径

### 初级 (1-2周)
1. 边缘计算概念
2. MQTT/CoAP协议
3. 树莓派/Arduino基础

### 中级 (2-3周)
1. 时序数据处理
2. 轻量级ML模型
3. 设备管理

### 高级 (2-4周)
1. 边缘AI推理
2. 数字孪生
3. 工业IoT

### 专家级 (持续)
1. 5G边缘计算
2. 边缘协同学习
3. 雾计算架构

## 参考资料

### 协议文档
- [MQTT官方文档](https://mqtt.org/)
- [CoAP RFC 7252](https://tools.ietf.org/html/rfc7252)

### 平台
- [AWS IoT Greengrass](https://aws.amazon.com/greengrass/)
- [Azure IoT Edge](https://azure.microsoft.com/services/iot-edge/)

---

**知识ID**: `edge-iot-complete`  
**领域**: edge-iot  
**类型**: standards  
**难度**: intermediate  
**质量分**: 92  
**维护者**: iot-team@superdev.com  
**最后更新**: 2026-03-28
