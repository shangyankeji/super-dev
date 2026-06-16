---
title: 移动开发反模式库
category: mobile/antipatterns
version: 1.0.0
last_updated: 2026-03-20
maintainer: Excellent（11964948@qq.com）
knowledge_score: 9.0/10
domain: mobile
difficulty: intermediate
quality_score: 70
---

# 开发：Excellent（11964948@qq.com）

## 移动开发反模式库

### 1. 性能反模式

#### 1.1 长列表使用 ScrollView
**反模式描述**
使用 ScrollView 渲染长列表（100+ 项），导致内存占用过高和滚动卡顿。

**识别信号**
- 列表项超过 50 个仍使用 ScrollView
- 滚动时 FPS 低于 50
- 内存占用持续增长

**替代实践**
使用虚拟列表（FlatList/ListView.builder）只渲染可见区域。

```typescript
// 反模式
<ScrollView>
  {items.map(item => <Item key={item.id} data={item} />)}
</ScrollView>

// 正确做法
<FlatList
  data={items}
  renderItem={({ item }) => <Item data={item} />}
  keyExtractor={item => item.id}
  initialNumToRender={10}
  maxToRenderPerBatch={10}
  windowSize={5}
/>
```

#### 1.2 主线程执行耗时操作
**反模式描述**
在主线程执行网络请求、大量计算、文件读写，导致 ANR 或卡顿。

**识别信号**
- UI 线程阻塞超过 16ms
- 用户操作响应延迟
- ANR 率 > 0.05%

**替代实践**
使用异步任务或后台线程。

```dart
// 反模式
void loadData() {
  final data = heavyComputation(); // 阻塞主线程
  setState(() {
    items = data;
  });
}

// 正确做法
Future<void> loadData() async {
  final data = await compute(heavyComputation, null); // 后台线程
  setState(() {
    items = data;
  });
}
```

#### 1.3 大图片未压缩直接加载
**反模式描述**
加载原图（5MB+）到内存，导致 OOM 和加载缓慢。

**识别信号**
- 内存峰值超过 300MB
- 图片加载超过 2 秒
- OOM 崩溃率高

**替代实践**
压缩图片并使用缓存策略。

```typescript
// 反模式
<Image source={{ uri: originalUrl }} style={{ width: 100, height: 100 }} />

// 正确做法
<FastImage
  source={{
    uri: thumbnailUrl,
    priority: FastImage.priority.normal,
  }}
  style={{ width: 100, height: 100 }}
  resizeMode={FastImage.resizeMode.cover}
/>
```

### 2. 架构反模式

#### 2.1 业务逻辑写在视图层
**反模式描述**
将复杂业务规则直接写在组件中，导致组件臃肿、难以测试和维护。

**识别信号**
- 组件代码超过 300 行
- 组件包含多个 API 调用和复杂计算
- 单元测试困难

**替代实践**
将业务逻辑抽离到 Service 层或 ViewModel。

```typescript
// 反模式
const UserList = () => {
  const [users, setUsers] = useState([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    setLoading(true);
    fetch('/api/users')
      .then(res => res.json())
      .then(data => {
        const filtered = data.filter(u => u.active);
        const sorted = filtered.sort((a, b) => b.score - a.score);
        setUsers(sorted);
        setLoading(false);
      });
  }, []);

  return <FlatList data={users} ... />;
};

// 正确做法
// userService.ts
export class UserService {
  async getActiveUsersSortedByScore(): Promise<User[]> {
    const response = await fetch('/api/users');
    const users = await response.json();
    return users
      .filter(u => u.active)
      .sort((a, b) => b.score - a.score);
  }
}

// UserList.tsx
const UserList = () => {
  const [users, setUsers] = useState([]);
  const userService = new UserService();

  useEffect(() => {
    userService.getActiveUsersSortedByScore()
      .then(setUsers);
  }, []);

  return <FlatList data={users} ... />;
};
```

#### 2.2 全局状态滥用
**反模式描述**
将所有状态都放入全局状态管理，导致性能下降和调试困难。

**识别信号**
- 全局 store 包含大量临时状态
- 组件重渲染频繁
- 状态更新链路复杂

**替代实践**
本地状态优先，仅在需要跨组件共享时使用全局状态。

```typescript
// 反模式
const store = {
  modalVisible: false,  // 仅在一个组件使用
  inputValue: '',       // 仅在一个组件使用
  theme: 'dark',        // 全局共享，合理
};

// 正确做法
const Component = () => {
  const [modalVisible, setModalVisible] = useState(false);
  const [inputValue, setInputValue] = useState('');
  const theme = useThemeStore(state => state.theme);  // 仅全局状态
};
```

### 3. UI 反模式

#### 3.1 硬编码样式和颜色
**反模式描述**
在代码中直接写死颜色值和样式，导致主题切换困难和维护成本高。

**识别信号**
- 代码中出现大量 #FFFFFF、#000000
- 同一颜色在不同文件重复定义
- 主题切换需求无法实现

**替代实践**
使用设计令牌（Design Tokens）和主题系统。

```typescript
// 反模式
<View style={{ backgroundColor: '#1E88E5', padding: 16 }}>
  <Text style={{ color: '#FFFFFF', fontSize: 16 }}>Title</Text>
</View>

// 正确做法
// theme.ts
export const theme = {
  colors: {
    primary: '#1E88E5',
    text: {
      primary: '#FFFFFF',
      secondary: '#757575',
    },
  },
  spacing: {
    md: 16,
  },
  typography: {
    body: {
      fontSize: 16,
    },
  },
};

// Component.tsx
<View style={{ backgroundColor: theme.colors.primary, padding: theme.spacing.md }}>
  <Text style={{ color: theme.colors.text.primary, fontSize: theme.typography.body.fontSize }}>
    Title
  </Text>
</View>
```

#### 3.2 缺少加载和错误状态
**反模式描述**
只处理正常状态，缺少加载中、空态、错误态处理。

**识别信号**
- 网络慢时界面空白
- 请求失败无提示
- 空数据显示异常布局

**替代实践**
完整的状态矩阵处理。

```dart
// 反模式
Widget build(BuildContext context) {
  return ListView(children: items);
}

// 正确做法
Widget build(BuildContext context) {
  if (isLoading) {
    return const Center(child: CircularProgressIndicator());
  }

  if (error != null) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          const Icon(Icons.error_outline, size: 48),
          const SizedBox(height: 16),
          Text(error!),
          ElevatedButton(
            onPressed: retry,
            child: const Text('重试'),
          ),
        ],
      ),
    );
  }

  if (items.isEmpty) {
    return const Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(Icons.inbox, size: 48),
          SizedBox(height: 16),
          Text('暂无数据'),
        ],
      ),
    );
  }

  return ListView(children: items);
}
```

### 4. 网络反模式

#### 4.1 网络请求无超时和重试
**反模式描述**
网络请求不设置超时时间，失败后不重试，用户体验差。

**识别信号**
- 弱网环境请求卡死
- 一次失败即展示错误
- 用户频繁手动重试

**替代实践**
设置合理超时和重试策略。

```typescript
// 反模式
const response = await fetch('/api/data');
const data = await response.json();

// 正确做法
async function fetchWithRetry(
  url: string,
  options: RequestInit = {},
  maxRetries = 3
): Promise<Response> {
  const timeout = 10000; // 10 秒超时

  for (let i = 0; i < maxRetries; i++) {
    try {
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), timeout);

      const response = await fetch(url, {
        ...options,
        signal: controller.signal,
      });

      clearTimeout(timeoutId);

      if (response.ok) {
        return response;
      }
    } catch (error) {
      if (i === maxRetries - 1) throw error;
      await new Promise(resolve => setTimeout(resolve, 1000 * (i + 1)));
    }
  }

  throw new Error('Network request failed after retries');
}
```

#### 4.2 缺少离线缓存
**反模式描述**
每次打开应用都从网络加载数据，离线时无法使用。

**识别信号**
- 无网络时应用不可用
- 重复加载相同数据浪费流量
- 加载速度慢

**替代实践**
实施离线优先策略（Cache-First）。

```dart
// 反模式
Future<List<Item>> fetchItems() async {
  final response = await dio.get('/api/items');
  return (response.data as List).map((json) => Item.fromJson(json)).toList();
}

// 正确做法
Future<List<Item>> fetchItems() async {
  // 1. 先读取本地缓存
  final cached = await localDb.getItems();
  if (cached.isNotEmpty) {
    // 立即返回缓存数据
    _notifyListeners(cached);
  }

  // 2. 后台请求网络数据
  try {
    final response = await dio.get('/api/items');
    final items = (response.data as List)
        .map((json) => Item.fromJson(json))
        .toList();

    // 3. 更新本地缓存
    await localDb.saveItems(items);

    // 4. 通知更新
    _notifyListeners(items);
    return items;
  } catch (e) {
    // 网络失败，使用缓存（如果有）
    if (cached.isNotEmpty) {
      return cached;
    }
    rethrow;
  }
}
```

### 5. 安全反模式

#### 5.1 敏感数据明文存储
**反模式描述**
将 token、密码等敏感信息直接存储在 AsyncStorage 或 SharedPreferences。

**识别信号**
- 代码中使用 AsyncStorage 存储 token
- root 权限设备可直接读取敏感数据
- 数据泄露风险高

**替代实践**
使用系统安全存储（Keychain/Keystore）。

```typescript
// 反模式
await AsyncStorage.setItem('auth_token', token);

// 正确做法
import Keychain from 'react-native-keychain';

// 存储
await Keychain.setGenericPassword('auth', token);

// 读取
const credentials = await Keychain.getGenericPassword();
if (credentials) {
  const token = credentials.password;
}
```

#### 5.2 WebView 不安全配置
**反模式描述**
WebView 允许任意来源加载、开启 JavaScript 无限制、文件访问权限过度。

**识别信号**
- WebView 加载 file:// 协议
- 允许所有来源内容
- 无白名单限制

**替代实践**
严格限制 WebView 安全配置。

```typescript
// 反模式
<WebView
  source={{ uri: url }}
  javaScriptEnabled={true}
  originWhitelist={['*']}
/>

// 正确做法
<WebView
  source={{ uri: allowedDomains.includes(domain) ? url : 'about:blank' }}
  javaScriptEnabled={true}
  originWhitelist={['https://*.example.com']}
  mixedContentMode="never"
  allowFileAccess={false}
  allowUniversalAccessFromFileURLs={false}
  onShouldStartLoadWithRequest={(request) => {
    return allowedDomains.some(domain => request.url.includes(domain));
  }}
/>
```

### 6. 数据库反模式

#### 6.1 主线程数据库操作
**反模式描述**
在主线程执行数据库查询和写入，导致 UI 卡顿。

**识别信号**
- 数据库操作时界面冻结
- 数据量大时响应慢
- ANR 告警

**替代实践**
数据库操作移到后台线程。

```dart
// 反模式
Future<void> saveItems(List<Item> items) async {
  final db = await database;
  for (final item in items) {
    await db.insert('items', item.toMap()); // 同步插入，阻塞主线程
  }
}

// 正确做法
Future<void> saveItems(List<Item> items) async {
  final db = await database;
  await db.transaction((txn) async {
    final batch = txn.batch();
    for (final item in items) {
      batch.insert('items', item.toMap());
    }
    await batch.commit(); // 批量提交，后台执行
  });
}
```

#### 6.2 缺少数据库索引
**反模式描述**
高频查询字段未建立索引，导致查询性能差。

**识别信号**
- 查询耗时 > 100ms
- 数据库 CPU 占用高
- 查询全表扫描

**替代实践**
为查询字段创建索引。

```sql
-- 反模式
CREATE TABLE orders (
  id INTEGER PRIMARY KEY,
  user_id INTEGER,
  status TEXT,
  created_at INTEGER
);

-- 查询慢
SELECT * FROM orders WHERE user_id = ? AND status = ?;

-- 正确做法
CREATE TABLE orders (
  id INTEGER PRIMARY KEY,
  user_id INTEGER,
  status TEXT,
  created_at INTEGER
);

-- 创建索引
CREATE INDEX idx_orders_user_status ON orders(user_id, status);
CREATE INDEX idx_orders_created ON orders(created_at);
```

### 7. 测试反模式

#### 7.1 测试依赖真实后端
**反模式描述**
单元测试依赖真实 API，导致测试不稳定、速度慢。

**识别信号**
- 测试失败因网络问题
- 测试运行时间 > 1 分钟
- CI 环境测试频繁失败

**替代实践**
使用 Mock 数据和 Mock Server。

```typescript
// 反模式
test('fetch users', async () => {
  const users = await apiClient.getUsers(); // 依赖真实 API
  expect(users.length).toBeGreaterThan(0);
});

// 正确做法
import { rest } from 'msw';
import { setupServer } from 'msw/node';

const server = setupServer(
  rest.get('/api/users', (req, res, ctx) => {
    return res(ctx.json([
      { id: 1, name: 'User 1' },
      { id: 2, name: 'User 2' },
    ]));
  })
);

beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

test('fetch users', async () => {
  const users = await apiClient.getUsers();
  expect(users).toHaveLength(2);
  expect(users[0].name).toBe('User 1');
});
```

#### 7.2 缺少边界条件测试
**反模式描述**
只测试正常场景，缺少边界条件和异常场景测试。

**识别信号**
- 测试用例全是 happy path
- 线上频繁出现未测试场景的 bug
- 测试覆盖率虚高但质量低

**替代实践**
完整的测试场景覆盖。

```dart
// 反模式
test('calculate discount', () {
  expect(calculateDiscount(100, 0.1), equals(90));
});

// 正确做法
group('calculateDiscount', () {
  test('normal discount', () {
    expect(calculateDiscount(100, 0.1), equals(90));
  });

  test('zero discount', () {
    expect(calculateDiscount(100, 0), equals(100));
  });

  test('full discount', () {
    expect(calculateDiscount(100, 1), equals(0));
  });

  test('negative price throws', () {
    expect(() => calculateDiscount(-100, 0.1), throwsArgumentError);
  });

  test('invalid discount throws', () {
    expect(() => calculateDiscount(100, 1.5), throwsArgumentError);
  });
});
```

### 8. 内存管理反模式

#### 8.1 监听器未取消订阅
**反模式描述**
订阅事件流或监听器后未取消，导致内存泄漏和意外行为。

**识别信号**
- 页面关闭后仍收到事件
- 内存占用持续增长
- 多次进入同一页面监听器重复

**替代实践**
在组件卸载时取消订阅。

```typescript
// 反模式
useEffect(() => {
  eventEmitter.addListener('userUpdate', handleUserUpdate);
}, []);

// 正确做法
useEffect(() => {
  const subscription = eventEmitter.addListener('userUpdate', handleUserUpdate);

  return () => {
    subscription.remove(); // 组件卸载时取消订阅
  };
}, []);
```

#### 8.2 闭包引用导致对象无法释放
**反模式描述**
闭包持有大对象引用，导致无法被垃圾回收。

**识别信号**
- 内存分析显示大量 Detached DOM
- 重复操作后内存增长
- WeakRef 对象未被回收

**替代实践**
及时释放引用或使用 WeakMap。

```dart
// 反模式
class DataService {
  List<Data> _cache = [];

  void fetchData(Function(List<Data>) callback) {
    loadFromNetwork().then((data) {
      _cache.addAll(data); // 持续累积
      callback(data);
    });
  }
}

// 正确做法
class DataService {
  final int _maxCacheSize = 100;
  List<Data> _cache = [];

  void fetchData(Function(List<Data>) callback) {
    loadFromNetwork().then((data) {
      _cache.addAll(data);

      // 限制缓存大小
      if (_cache.length > _maxCacheSize) {
        _cache = _cache.sublist(_cache.length - _maxCacheSize);
      }

      callback(data);
    });
  }
}
```

### 9. 生命周期反模式

#### 9.1 组件卸载后更新状态
**反模式描述**
异步操作完成后更新已卸载组件的状态，导致内存泄漏警告。

**识别信号**
- 控制台警告：Can't perform a React state update on an unmounted component
- 内存泄漏警告
- 应用行为异常

**替代实践**
使用取消标志或 AbortController。

```typescript
// 反模式
useEffect(() => {
  fetchUserData().then(setUser);
}, []);

// 正确做法
useEffect(() => {
  let isCancelled = false;

  fetchUserData().then(user => {
    if (!isCancelled) {
      setUser(user);
    }
  });

  return () => {
    isCancelled = true;
  };
}, []);
```

#### 9.2 未处理 Android Activity 重建
**反模式描述**
未处理 Android 配置变更（旋转、分屏）导致的 Activity 重建，状态丢失。

**识别信号**
- 旋转屏幕后数据丢失
- 分屏切换后崩溃
- 后台恢复后状态异常

**替代实践**
使用状态保存和恢复机制。

```dart
// 反模式
class _MyWidgetState extends State<MyWidget> {
  late Data data;

  @override
  void initState() {
    super.initState();
    data = loadData(); // 重建时丢失
  }
}

// 正确做法
class _MyWidgetState extends State<MyWidget> {
  late Data data;

  @override
  void initState() {
    super.initState();
    data = loadData();
  }

  // 保存状态
  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.paused) {
      saveState(data);
    }
  }

  // 恢复状态
  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    final savedData = getSavedState();
    if (savedData != null) {
      data = savedData;
    }
  }
}
```

### 参考资料
- React Native 性能优化：https://reactnative.dev/docs/performance
- Flutter 性能最佳实践：https://flutter.dev/docs/performance/best-practices
- Android 性能模式：https://developer.android.com/topic/performance
- iOS 性能优化：https://developer.apple.com/documentation/performance
- OWASP 移动安全：https://owasp.org/www-project-mobile-security/