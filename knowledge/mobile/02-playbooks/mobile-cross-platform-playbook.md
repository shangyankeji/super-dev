---
id: mobile-cross-platform-playbook
title: 跨平台移动开发实战手册（React Native / Flutter）
domain: mobile
category: 02-playbooks
difficulty: advanced
tags: [mobile, react-native, flutter, dart, performance, native-modules, expo, enterprise, ios, android, cross-platform]
quality_score: 93
maintainer: mobile-team@superdev.com
last_updated: 2026-06-15
---

# 跨平台移动开发实战手册

> 基于 [Flutter vs React Native 2025 完整对比](https://www.alimertgulec.com/en/blog/flutter-vs-react-native-2025) + [Nomtek 2025 分析](https://www.nomtek.com/blog/flutter-vs-react-native) + [Foresight Mobile](https://foresightmobile.com/blog/why-flutter-outperforms-the-competition)

## React Native vs Flutter 决策

| 维度 | React Native | Flutter |
|------|-------------|---------|
| 语言 | JavaScript/TypeScript | Dart |
| 渲染 | 原生组件（新架构 Fabric） | 自绘引擎（Skia/Impeller） |
| 性能 | 接近原生（新架构提升大） | 一致高性能（编译 ARM） |
| 生态 | npm 生态（最大） | pub.dev（快速增长） |
| 团队 | JS/Web 团队上手快 | 需学 Dart |
| 热更新 | CodePush（JS 可热更） | 不支持（需重新发版） |
| 桌面/Web | 社区方案 | 官方支持 |
| UI 一致性 | 各平台外观不同 | 像素级一致 |
| 企业案例 | Facebook/Instagram/Discord | Google/Alibaba/ByteDance |

**选 React Native 当：** 团队是 JS 栈、需要 CodePush 热更新、已有 Web 代码复用
**选 Flutter 当：** 需要像素级 UI 一致、强类型安全、单代码库覆盖 iOS/Android/Web/桌面

## React Native 最佳实践

### 新架构（Fabric + TurboModules）
```javascript
// React Native 0.76+ 默认开启新架构
// app.json
{
  "expo": {
    "newArchEnabled": true  // Expo SDK 52+
  }
}

// 新架构收益：
// - Fabric：同步渲染，更流畅动画
// - TurboModules：懒加载原生模块，启动更快
// - 不再需要 Bridge（JSI 直接调用）
```

### 性能优化
```typescript
// ❌ 内联函数 + 对象导致 FlatList 全量重渲染
<FlatList
  data={items}
  renderItem={({ item }) => <Item onPress={() => handlePress(item)} data={{...item}} />}
/>

// ✅ useCallback + useMemo + React.memo
const renderItem = useCallback(({ item }) => (
  <MemoItem onPress={handlePress} item={item} />
), [handlePress]);

const keyExtractor = useCallback((item) => item.id, []);

<FlatList
  data={items}
  renderItem={renderItem}
  keyExtractor={keyExtractor}
  removeClippedSubviews={true}        // 卸载屏幕外的组件
  maxToRenderPerBatch={10}             // 批量渲染
  windowSize={5}                       // 渲染窗口
/>
```

### Expo（推荐工作流）
```bash
# Expo 提供 OTA 热更新 + 云构建
npx expo prebuild          # 生成原生代码（需要自定义原生模块时）
eas build --platform ios   # 云端构建（不需要 Mac！）
eas update                 # OTA 热更新（不用过 App Store 审核）
```

## Flutter 最佳实践

### Widget 性能
```dart
// ❌ 整个 ListView 在 setState 时全量重建
class TodoList extends StatefulWidget {
  State createState() => _TodoListState();
}
class _TodoListState extends State<TodoList> {
  List<Todo> todos = [];
  void toggle(int id) {
    setState(() { todos = todos.map((t) => ...).toList(); }); // 全量重建！
  }
}

// ✅ const Widget + 局部重建
class TodoItem extends StatelessWidget {
  const TodoItem({super.key, required this.todo, required this.onToggle});
  final Todo todo;
  final VoidCallback onToggle;
  Widget build(BuildContext context) {
    return ListTile(
      leading: Checkbox(value: todo.done, onChanged: (_) => onToggle()),
      title: Text(todo.title),
    );
  }
}
// 用 const 构造器 → Flutter 跳过未变 Widget 的重建
```

### 状态管理（Riverpod 推荐）
```dart
// ✅ Riverpod（类型安全、可测试）
final todoProvider = StateNotifierProvider<TodoNotifier, List<Todo>>((ref) {
  return TodoNotifier();
});

class TodoNotifier extends StateNotifier<List<Todo>> {
  TodoNotifier() : super([]);
  void toggle(String id) {
    state = [
      for (final t in state)
        if (t.id == id) t.copyWith(done: !t.done) else t,
    ];
  }
}

// Widget 消费
class TodoList extends ConsumerWidget {
  Widget build(BuildContext context, WidgetRef ref) {
    final todos = ref.watch(todoProvider);
    return ListView.builder(
      itemCount: todos.length,
      itemBuilder: (_, i) => TodoItem(
        todo: todos[i],
        onToggle: () => ref.read(todoProvider.notifier).toggle(todos[i].id),
      ),
    );
  }
}
```

## 通用移动最佳实践

### 启动优化
| 技术 | RN | Flutter |
|------|-----|---------|
| 懒加载模块 | React.lazy + Splash | deferred imports |
| 预取关键数据 | useEffect 首屏前请求 | FutureBuilder |
| 减少首屏 Widget | 分屏加载 | 分页构建 |
| 原生启动优化 | Hermes 引擎 | 预编译 AOT |

### 安全
- [ ] API 密钥不硬编码（用原生 Keychain/Keystore）
- [ ] HTTPS + 证书绑定（SSL pinning）
- [ ] 敏感数据加密存储
- [ ] 代码混淆（防逆向）
- [ ] Root/越狱检测（银行/支付应用）
- [ ] 深链接验证（防 Deeplink 劫持）

## 生产检查清单
- [ ] 启动时间 < 2s（冷启动）
- [ ] 60fps 滚动/动画
- [ ] 列表虚拟化（FlatList/ListView.builder）
- [ ] 图片缓存 + 懒加载
- [ ] 离线支持（本地缓存 + 同步）
- [ ] OTA 热更新（RN CodePush / 无 Flutter）
- [ ] Crash 上报（Sentry/Firebase Crashlytics）
- [ ] 深链接处理
- [ ] 推送通知
- [ ] App Store / Google Play 合规
