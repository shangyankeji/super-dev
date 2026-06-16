---
title: 移动应用性能优化手册
category: mobile/playbooks
version: 1.0.0
last_updated: 2026-03-20
maintainer: Excellent（11964948@qq.com）
knowledge_score: 9.0/10
domain: mobile
difficulty: intermediate
quality_score: 70
---

# 开发：Excellent（11964948@qq.com）

## 移动应用性能优化手册

### 执行步骤

#### 1. 性能基线建立
- 定义关键指标：启动时间、FPS、内存占用、包体积。
- 建立性能测试环境和监控工具。
- 收集不同设备的性能数据作为基线。
- 设置性能预算和告警阈值。

#### 2. 性能瓶颈定位
- 使用性能分析工具（Xcode Instruments、Android Profiler、Flipper）。
- 分析启动流程、渲染性能、内存使用。
- 定位网络请求、数据库查询、图片加载瓶颈。
- 识别主线程阻塞和 ANR（Application Not Responding）。

#### 3. 启动时间优化
- 延迟加载非关键模块。
- 优化初始化流程，避免同步阻塞。
- 减少主线程任务，使用后台线程。
- 预加载关键资源和数据。
- 目标：冷启动 < 2 秒，热启动 < 0.5 秒。

#### 4. UI 渲染优化
- 保持 60 FPS，避免卡顿（每帧 < 16.67ms）。
- 减少视图层级，避免过度绘制。
- 使用虚拟列表渲染长列表。
- 优化图片加载和缓存策略。
- 避免在滚动时执行耗时操作。

#### 5. 内存优化
- 及时释放不再使用的资源。
- 避免内存泄漏（监听器未取消、闭包引用）。
- 优化图片内存占用（压缩、采样、复用）。
- 使用对象池复用对象。
- 监控内存峰值和 OOM 崩溃。

#### 6. 网络优化
- 减少 HTTP 请求次数（合并、批量）。
- 启用 Gzip 压缩和 HTTPS。
- 使用 CDN 加速静态资源。
- 实施离线缓存策略（Cache-First、Network-First）。
- 预加载和懒加载结合。

#### 7. 包体积优化
- 压缩图片资源（WebP、TinyPNG）。
- 移除未使用的代码和资源。
- 启用代码混淆和 Tree Shaking。
- 按需加载功能模块。
- 目标：Android APK < 50MB，iOS IPA < 100MB。

#### 8. 数据库优化
- 创建合适的索引加速查询。
- 避免在大表上执行全表扫描。
- 批量插入和更新操作。
- 使用事务保证数据一致性。
- 定期清理过期数据。

#### 9. 电量优化
- 减少后台任务和轮询。
- 优化定位策略（精度、频率）。
- 避免频繁的网络请求和唤醒。
- 使用 JobScheduler 或 WorkManager 调度任务。
- 监控电量消耗。

#### 10. 持续监控与回归
- 建立性能监控平台（Firebase Performance、Sentry）。
- 设置性能门禁，失败自动阻断发布。
- 定期性能回归测试。
- 收集线上用户真实性能数据。

### 常见优化策略

#### React Native 优化
```typescript
/**
 * 开发：Excellent（11964948@qq.com）
 * 功能：优化 FlatList 性能
 * 作用：通过配置优化参数提升长列表渲染性能
 * 创建时间：2026-03-20
 * 最后修改：2026-03-20
 */

import React, { useCallback, useMemo } from 'react';
import { FlatList, View, Text, StyleSheet } from 'react-native';

interface Item {
  id: string;
  title: string;
}

const OptimizedList: React.FC<{ items: Item[] }> = ({ items }) => {
  // 使用 useCallback 避免重复创建函数
  const renderItem = useCallback(
    ({ item }: { item: Item }) => (
      <View style={styles.item}>
        <Text style={styles.title}>{item.title}</Text>
      </View>
    ),
    []
  );

  // 使用 useMemo 缓存计算结果
  const keyExtractor = useCallback(
    (item: Item) => item.id,
    []
  );

  // 使用 getItemLayout 提升性能（如果 item 高度固定）
  const getItemLayout = useCallback(
    (_: any, index: number) => ({
      length: 60,
      offset: 60 * index,
      index,
    }),
    []
  );

  return (
    <FlatList
      data={items}
      renderItem={renderItem}
      keyExtractor={keyExtractor}
      getItemLayout={getItemLayout}
      // 性能优化参数
      initialNumToRender={10}
      maxToRenderPerBatch={10}
      windowSize={5}
      removeClippedSubviews={true}
      updateCellsBatchingPeriod={50}
      onEndReachedThreshold={0.5}
    />
  );
};

const styles = StyleSheet.create({
  item: {
    height: 60,
    justifyContent: 'center',
    paddingHorizontal: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#e0e0e0',
  },
  title: {
    fontSize: 16,
  },
});

export default OptimizedList;
```

#### Flutter 优化
```dart
// 开发：Excellent（11964948@qq.com）
// 功能：优化 ListView 性能
// 作用：通过 const 和 builder 提升长列表渲染性能
// 创建时间：2026-03-20
// 最后修改：2026-03-20

import 'package:flutter/material.dart';

class Item {
  final String id;
  final String title;

  const Item({required this.id, required this.title});
}

class OptimizedList extends StatelessWidget {
  final List<Item> items;

  const OptimizedList({Key? key, required this.items}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return ListView.builder(
      itemCount: items.length,
      // 使用 const 提升性能
      itemBuilder: (context, index) {
        final item = items[index];
        return _ItemTile(item: item);
      },
      // 添加缓存扩展
      cacheExtent: 1000,
    );
  }
}

// 提取为独立的 StatelessWidget 并使用 const 构造函数
class _ItemTile extends StatelessWidget {
  final Item item;

  const _ItemTile({Key? key, required this.item}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Container(
      height: 60,
      padding: const EdgeInsets.symmetric(horizontal: 16),
      alignment: Alignment.centerLeft,
      decoration: const BoxDecoration(
        border: Border(
          bottom: BorderSide(color: Color(0xFFE0E0E0)),
        ),
      ),
      child: Text(
        item.title,
        style: const TextStyle(fontSize: 16),
      ),
    );
  }
}
```

### 性能监控代码示例

#### React Native 性能监控
```typescript
/**
 * 开发：Excellent（11964948@qq.com）
 * 功能：性能监控工具
 * 作用：记录和上报关键性能指标
 * 创建时间：2026-03-20
 * 最后修改：2026-03-20
 */

import { Platform, NativeModules } from 'react-native';

interface PerformanceMetric {
  name: string;
  duration: number;
  timestamp: number;
  metadata?: Record<string, any>;
}

class PerformanceMonitor {
  private static instance: PerformanceMonitor;
  private metrics: PerformanceMetric[] = [];

  static getInstance(): PerformanceMonitor {
    if (!PerformanceMonitor.instance) {
      PerformanceMonitor.instance = new PerformanceMonitor();
    }
    return PerformanceMonitor.instance;
  }

  // 记录启动时间
  markAppStart() {
    const startTime = Date.now();
    this.recordMetric('app_start', 0, { startTime });
    return startTime;
  }

  // 记录启动完成
  markAppReady(startTime: number) {
    const duration = Date.now() - startTime;
    this.recordMetric('app_ready', duration);
    this.uploadMetrics();
  }

  // 记录屏幕渲染时间
  markScreenRender(screenName: string, duration: number) {
    this.recordMetric('screen_render', duration, { screenName });
  }

  // 记录 API 请求时间
  markApiRequest(endpoint: string, duration: number, success: boolean) {
    this.recordMetric('api_request', duration, {
      endpoint,
      success,
    });
  }

  private recordMetric(
    name: string,
    duration: number,
    metadata?: Record<string, any>
  ) {
    this.metrics.push({
      name,
      duration,
      timestamp: Date.now(),
      metadata,
    });
  }

  // 上报性能数据
  private async uploadMetrics() {
    if (this.metrics.length === 0) return;

    try {
      // 上报到监控平台
      await fetch('https://monitoring.example.com/api/performance', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          platform: Platform.OS,
          version: Platform.Version,
          metrics: this.metrics,
        }),
      });

      this.metrics = [];
    } catch (error) {
      console.error('Failed to upload performance metrics:', error);
    }
  }
}

export const performanceMonitor = PerformanceMonitor.getInstance();
```

#### Flutter 性能监控
```dart
// 开发：Excellent（11964948@qq.com）
// 功能：性能监控工具
// 作用：记录和上报关键性能指标
// 创建时间：2026-03-20
// 最后修改：2026-03-20

import 'dart:async';
import 'dart:io';
import 'package:flutter/foundation.dart';
import 'package:dio/dio.dart';

class PerformanceMetric {
  final String name;
  final int duration;
  final int timestamp;
  final Map<String, dynamic>? metadata;

  PerformanceMetric({
    required this.name,
    required this.duration,
    required this.timestamp,
    this.metadata,
  });

  Map<String, dynamic> toJson() => {
        'name': name,
        'duration': duration,
        'timestamp': timestamp,
        'metadata': metadata,
      };
}

class PerformanceMonitor {
  static final PerformanceMonitor _instance = PerformanceMonitor._internal();
  factory PerformanceMonitor() => _instance;
  PerformanceMonitor._internal();

  final List<PerformanceMetric> _metrics = [];
  final Dio _dio = Dio();

  // 记录启动时间
  int markAppStart() {
    final startTime = DateTime.now().millisecondsSinceEpoch;
    _recordMetric('app_start', 0, {'startTime': startTime});
    return startTime;
  }

  // 记录启动完成
  void markAppReady(int startTime) {
    final duration = DateTime.now().millisecondsSinceEpoch - startTime;
    _recordMetric('app_ready', duration);
    _uploadMetrics();
  }

  // 记录屏幕渲染时间
  void markScreenRender(String screenName, int duration) {
    _recordMetric('screen_render', duration, {'screenName': screenName});
  }

  // 记录 API 请求时间
  void markApiRequest(String endpoint, int duration, bool success) {
    _recordMetric('api_request', duration, {
      'endpoint': endpoint,
      'success': success,
    });
  }

  void _recordMetric(
    String name,
    int duration, [
    Map<String, dynamic>? metadata,
  ]) {
    _metrics.add(PerformanceMetric(
      name: name,
      duration: duration,
      timestamp: DateTime.now().millisecondsSinceEpoch,
      metadata: metadata,
    ));
  }

  // 上报性能数据
  Future<void> _uploadMetrics() async {
    if (_metrics.isEmpty) return;

    try {
      await _dio.post(
        'https://monitoring.example.com/api/performance',
        data: {
          'platform': Platform.operatingSystem,
          'version': Platform.version,
          'metrics': _metrics.map((m) => m.toJson()).toList(),
        },
      );
      _metrics.clear();
    } catch (e) {
      debugPrint('Failed to upload performance metrics: $e');
    }
  }
}

final performanceMonitor = PerformanceMonitor();
```

### 性能优化检查清单

#### 启动性能
- [ ] 冷启动时间 < 2 秒
- [ ] 热启动时间 < 0.5 秒
- [ ] 延迟加载非关键模块
- [ ] 优化初始化流程
- [ ] 预加载关键数据

#### UI 渲染性能
- [ ] FPS 稳定在 60 帧
- [ ] 避免过度绘制（Overdraw < 2x）
- [ ] 使用虚拟列表渲染长列表
- [ ] 优化图片加载策略
- [ ] 避免在滚动时执行耗时操作

#### 内存优化
- [ ] 内存峰值 < 200MB
- [ ] 无内存泄漏
- [ ] 及时释放资源
- [ ] 优化图片内存占用
- [ ] 监控 OOM 崩溃率

#### 网络性能
- [ ] 首屏数据加载 < 1 秒
- [ ] 启用 Gzip 压缩
- [ ] 实施 CDN 加速
- [ ] 离线缓存策略
- [ ] 减少请求次数

#### 包体积
- [ ] Android APK < 50MB
- [ ] iOS IPA < 100MB
- [ ] 压缩图片资源
- [ ] 移除未使用代码
- [ ] 启用代码混淆

### 参考资料
- React Native 性能优化：https://reactnative.dev/docs/performance
- Flutter 性能优化：https://flutter.dev/docs/performance/rendering
- Android 性能优化：https://developer.android.com/topic/performance
- iOS 性能优化：https://developer.apple.com/documentation/performance