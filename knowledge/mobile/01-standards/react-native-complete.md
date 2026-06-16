---
title: React Native 完整开发标准
category: mobile/standards
version: 1.0.0
last_updated: 2026-03-20
maintainer: Excellent（11964948@qq.com）
knowledge_score: 9.2/10
domain: mobile
difficulty: intermediate
quality_score: 70
---

# 开发：Excellent（11964948@qq.com）

## React Native 完整开发标准

### 1. 架构与分层
- 业务层、展示层、数据层严格分离。
- 使用 Redux Toolkit 或 Zustand 管理全局状态，避免 prop drilling。
- 网络请求封装统一拦截器（认证、错误处理、日志）。
- 导航使用 React Navigation 6+，支持深度链接。

### 2. 组件设计原则
- 单一职责：组件代码不超过 300 行。
- 可复用组件与业务组件分离。
- 使用 TypeScript 定义 Props 和 State 类型。
- 避免内联样式，使用 StyleSheet.create 或 styled-components。

### 3. 性能优化策略
- 使用 FlatList 替代 ScrollView 渲染长列表。
- 图片使用 react-native-fast-image 并配置缓存策略。
- 实施 Hermes 引擎优化启动时间。
- 避免不必要的重渲染，使用 React.memo 和 useMemo。

### 4. 原生模块集成
- 复杂功能使用原生模块（相机、地图、支付）。
- 遵循 iOS/Android 平台设计规范。
- 处理好权限申请流程（相机、定位、存储）。
- 原生模块提供 Promise 或 Event Emitter 接口。

### 5. 状态管理最佳实践
- 本地状态用 useState，共享状态用全局状态管理。
- 异步操作使用 Redux Toolkit createAsyncThunk。
- 避免在渲染函数中触发状态更新。
- 状态持久化使用 AsyncStorage 或 MMKV。

### 6. 错误处理与监控
- 全局错误边界捕获 React 错误。
- 网络错误统一处理并展示用户友好提示。
- 集成 Crashlytics 或 Sentry 崩溃监控。
- 关键路径埋点监控成功率和耗时。

### 7. 测试策略
- 单元测试：工具函数、业务逻辑（Jest）。
- 组件测试：使用 @testing-library/react-native。
- E2E 测试：使用 Detox 覆盖关键流程。
- 测试覆盖率 >= 70%。

### 8. 安全规范
- 敏感数据使用 Keychain（iOS）或 Keystore（Android）。
- 禁止硬编码密钥，使用 react-native-config。
- HTTPS 强制开启，证书校验严格。
- 代码混淆和加固（ProGuard/R8）。

### 9. 国际化与适配
- 使用 react-native-localize 获取系统语言。
- 文案抽取为多语言资源文件。
- 适配不同屏幕尺寸（PixelRatio）。
- 支持 RTL（阿拉伯语）布局。

### 10. 升级与维护
- 锁定 React Native 版本，避免频繁升级。
- 依赖库版本锁定，使用 lock 文件。
- 定期更新安全补丁。
- 使用 React Native Upgrade Helper 辅助升级。

## 代码示例

### TypeScript 组件示例
```typescript
/**
 * 开发：Excellent（11964948@qq.com）
 * 功能：用户头像组件
 * 作用：展示用户头像，支持默认头像和加载状态
 * 创建时间：2026-03-20
 * 最后修改：2026-03-20
 */

import React from 'react';
import { Image, View, StyleSheet, ActivityIndicator } from 'react-native';
import { COLORS } from '../constants/colors';

interface UserAvatarProps {
  uri?: string;
  size?: number;
  isLoading?: boolean;
}

export const UserAvatar: React.FC<UserAvatarProps> = ({
  uri,
  size = 48,
  isLoading = false
}) => {
  const containerStyle = {
    width: size,
    height: size,
    borderRadius: size / 2,
  };

  if (isLoading) {
    return (
      <View style={[styles.container, containerStyle, styles.loadingContainer]}>
        <ActivityIndicator size="small" color={COLORS.primary} />
      </View>
    );
  }

  return (
    <Image
      source={{ uri: uri || 'https://via.placeholder.com/150' }}
      style={[styles.container, containerStyle]}
      defaultSource={require('../assets/default-avatar.png')}
    />
  );
};

const styles = StyleSheet.create({
  container: {
    backgroundColor: COLORS.background,
  },
  loadingContainer: {
    justifyContent: 'center',
    alignItems: 'center',
  },
});
```

### 网络请求封装示例
```typescript
/**
 * 开发：Excellent（11964948@qq.com）
 * 功能：API 客户端封装
 * 作用：统一管理网络请求，包含认证和错误处理
 * 创建时间：2026-03-20
 * 最后修改：2026-03-20
 */

import AsyncStorage from '@react-native-async-storage/async-storage';

const BASE_URL = 'https://api.example.com';

interface RequestConfig {
  method?: 'GET' | 'POST' | 'PUT' | 'DELETE';
  headers?: Record<string, string>;
  body?: any;
}

class ApiClient {
  private async getAuthToken(): Promise<string | null> {
    return AsyncStorage.getItem('auth_token');
  }

  async request<T>(endpoint: string, config: RequestConfig = {}): Promise<T> {
    const token = await this.getAuthToken();

    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      ...config.headers,
    };

    if (token) {
      headers['Authorization'] = `Bearer ${token}`;
    }

    const response = await fetch(`${BASE_URL}${endpoint}`, {
      method: config.method || 'GET',
      headers,
      body: config.body ? JSON.stringify(config.body) : undefined,
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.message || 'Network request failed');
    }

    return response.json();
  }

  get<T>(endpoint: string) {
    return this.request<T>(endpoint);
  }

  post<T>(endpoint: string, body: any) {
    return this.request<T>(endpoint, { method: 'POST', body });
  }
}

export const apiClient = new ApiClient();
```

### FlatList 优化示例
```typescript
/**
 * 开发：Excellent（11964948@qq.com）
 * 功能：用户列表组件
 * 作用：高性能渲染用户列表，支持下拉刷新和加载更多
 * 创建时间：2026-03-20
 * 最后修改：2026-03-20
 */

import React, { useCallback, useState } from 'react';
import { FlatList, View, Text, StyleSheet, RefreshControl } from 'react-native';
import { UserAvatar } from './UserAvatar';

interface User {
  id: string;
  name: string;
  email: string;
  avatar?: string;
}

interface UserListProps {
  users: User[];
  onRefresh?: () => Promise<void>;
  onLoadMore?: () => Promise<void>;
}

export const UserList: React.FC<UserListProps> = ({
  users,
  onRefresh,
  onLoadMore,
}) => {
  const [refreshing, setRefreshing] = useState(false);
  const [loadingMore, setLoadingMore] = useState(false);

  const handleRefresh = useCallback(async () => {
    if (!onRefresh) return;
    setRefreshing(true);
    await onRefresh();
    setRefreshing(false);
  }, [onRefresh]);

  const handleEndReached = useCallback(async () => {
    if (!onLoadMore || loadingMore) return;
    setLoadingMore(true);
    await onLoadMore();
    setLoadingMore(false);
  }, [onLoadMore, loadingMore]);

  const renderItem = useCallback(
    ({ item }: { item: User }) => (
      <View style={styles.itemContainer}>
        <UserAvatar uri={item.avatar} size={48} />
        <View style={styles.infoContainer}>
          <Text style={styles.name}>{item.name}</Text>
          <Text style={styles.email}>{item.email}</Text>
        </View>
      </View>
    ),
    []
  );

  const keyExtractor = useCallback((item: User) => item.id, []);

  return (
    <FlatList
      data={users}
      renderItem={renderItem}
      keyExtractor={keyExtractor}
      refreshControl={
        <RefreshControl refreshing={refreshing} onRefresh={handleRefresh} />
      }
      onEndReached={handleEndReached}
      onEndReachedThreshold={0.5}
      initialNumToRender={10}
      maxToRenderPerBatch={10}
      windowSize={5}
      removeClippedSubviews={true}
    />
  );
};

const styles = StyleSheet.create({
  itemContainer: {
    flexDirection: 'row',
    padding: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#e0e0e0',
  },
  infoContainer: {
    marginLeft: 12,
    justifyContent: 'center',
  },
  name: {
    fontSize: 16,
    fontWeight: '600',
  },
  email: {
    fontSize: 14,
    color: '#666',
    marginTop: 4,
  },
});
```

## 反模式识别

### 反模式列表
- 在 render 函数中创建新对象或函数导致重渲染。
- 使用 ScrollView 渲染长列表导致性能问题。
- 直接操作 DOM 或使用 ref 滥用。
- 状态更新触发无限循环。
- 未处理网络请求失败和超时。
- 在主线程执行大量计算。
- 图片未压缩和缓存导致内存泄漏。
- 未做权限检查直接调用原生模块。

## 检查清单

### 开发阶段
- [ ] TypeScript 类型定义完整且严格
- [ ] 组件拆分合理，单一职责
- [ ] 使用 FlatList 渲染列表，配置优化参数
- [ ] 图片使用 fast-image 并配置缓存
- [ ] 状态管理合理，避免过度共享
- [ ] 网络请求封装统一，包含错误处理
- [ ] 样式使用 StyleSheet.create，避免内联
- [ ] 敏感数据存储使用安全存储方案
- [ ] 权限申请流程完整，包含拒绝处理
- [ ] 关键路径添加埋点监控

### 测试阶段
- [ ] 单元测试覆盖率 >= 70%
- [ ] 关键流程 E2E 测试通过
- [ ] 不同设备尺寸适配测试
- [ ] 网络异常场景测试（断网、超时）
- [ ] 内存泄漏检测通过
- [ ] 性能指标符合基线（启动时间、FPS）

### 发布阶段
- [ ] 生产环境配置正确（API 地址、密钥）
- [ ] 代码混淆和加固完成
- [ ] 崩溃监控集成完成
- [ ] 版本号和构建号更新
- [ ] 更新日志编写完成
- [ ] 应用商店素材准备完成

## 参考资料
- React Native 官方文档：https://reactnative.dev/
- React Navigation 文档：https://reactnavigation.org/
- Redux Toolkit 文档：https://redux-toolkit.js.org/
- Detox E2E 测试：https://wix.github.io/Detox/