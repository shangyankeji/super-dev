---
id: mobile-glossary
title: Mobile Development Glossary
domain: mobile
category: 06-glossary
difficulty: intermediate
tags: [agent, checklist, glossary, mobile, 术语对比速查表, 概述]
quality_score: 70
last_updated: 2026-06-15
---
# Mobile Development Glossary

## 概述

移动开发术语表涵盖 iOS、Android 和跨平台开发中的核心概念、框架、工具和技术。本术语表按字母顺序排列，每个词条包含定义、平台归属、使用场景和关联术语，适用于移动端开发工程师、技术负责人和全栈工程师。

---

## A

### AAB (Android App Bundle)

**定义**：Google 推出的 Android 应用发布格式，替代传统 APK。Google Play 根据用户设备配置（屏幕密度、CPU 架构、语言）动态生成优化后的 APK，减小下载体积。

**平台**：Android

**关联术语**：APK、Dynamic Delivery、App Thinning

### ABI (Application Binary Interface)

**定义**：应用二进制接口，定义了应用与操作系统之间的二进制级别交互规范。Android 常见 ABI：arm64-v8a、armeabi-v7a、x86_64。

**平台**：Android / iOS

**关联术语**：NDK、Native Module

### ANR (Application Not Responding)

**定义**：Android 系统检测到应用主线程被阻塞超过阈值时弹出的无响应对话框。输入事件 5 秒无响应或 BroadcastReceiver 10 秒未完成会触发 ANR。

**平台**：Android

**解决方案**：将耗时操作移到后台线程，避免主线程 I/O 和复杂计算。

**关联术语**：Main Thread、Jank

### APK (Android Package Kit)

**定义**：Android 应用的打包分发格式，包含编译后的代码（DEX）、资源文件、清单文件和签名信息。

**平台**：Android

**关联术语**：AAB、DEX、ProGuard

### App Thinning

**定义**：Apple 的应用瘦身技术集合，包括 Slicing（按设备生成变体）、Bitcode（服务端优化编译）和 On-Demand Resources（按需下载资源）。

**平台**：iOS

**关联术语**：AAB、Bundle Size

### ARC (Automatic Reference Counting)

**定义**：Apple 的自动引用计数内存管理机制。编译器自动插入 retain/release 代码管理对象生命周期，开发者需注意循环引用（Retain Cycle）。

**平台**：iOS

**关联术语**：Retain Cycle、weak/unowned、GC

### ASO (App Store Optimization)

**定义**：应用商店优化，提升 App 在搜索结果中排名的策略。优化维度包括标题关键词、图标截图、描述文案、评分评论、下载量和本地化。

**平台**：iOS / Android

**关联术语**：Bundle Size、Launch Screen

---

## B

### Bridge

**定义**：在跨平台框架中，连接 JavaScript/Dart 层与原生平台层的通信机制。React Native 旧架构使用异步 Bridge（JSON 序列化），新架构使用 JSI 实现同步调用。

**平台**：跨平台（React Native）

**关联术语**：JSI、Turbo Modules、React Native

---

## C

### CocoaPods

**定义**：iOS/macOS 的依赖管理工具，使用 Podfile 声明依赖，从中心仓库下载并集成第三方库。正逐渐被 Swift Package Manager 取代。

**平台**：iOS

**关联术语**：SPM、Gradle

### CodePush

**定义**：微软提供的移动端热更新服务，允许 React Native 应用绑定更新 JS Bundle 和资源文件而无需重新提交应用商店审核。

**平台**：跨平台（React Native）

**注意**：Apple 和 Google 对热更新有政策限制，不允许改变应用核心功能。

**关联术语**：OTA、JS Bundle、Expo Updates

### Compose (Jetpack Compose)

**定义**：Google 推出的 Android 声明式 UI 框架，使用 Kotlin 代码构建 UI，替代传统 XML 布局。类似 iOS 的 SwiftUI。

**平台**：Android

**关联术语**：SwiftUI、Flutter、声明式 UI

### Core Data

**定义**：Apple 提供的对象图和持久化框架，支持数据建模、查询、关系管理和 iCloud 同步。底层通常使用 SQLite。

**平台**：iOS / macOS

**关联术语**：Room、Realm、SQLite

### Cross-Platform

**定义**：使用同一套代码编译或运行在多个平台（iOS + Android + Web）的开发方式。分为编译型跨平台（Flutter -> 原生代码）和运行时跨平台（React Native -> JSI 调用原生组件）。

**关联术语**：React Native、Flutter、KMP

---

## D

### Deep Link

**定义**：深度链接，通过 URL 直接打开应用内的特定页面或内容。分为三种类型：URI Scheme（自定义协议）、Universal Links（iOS）、App Links（Android）。

**使用场景**：营销推广、跨应用跳转、Web 到 App 引流。

**关联术语**：Universal Links、App Links、Deferred Deep Link

### DEX (Dalvik Executable)

**定义**：Android 平台上的可执行文件格式，由 Java/Kotlin 字节码转换而来。ART 在安装时将 DEX 编译为本地机器码（AOT 编译）。

**平台**：Android

**关联术语**：ART、APK、R8

---

## E

### Expo

**定义**：React Native 的开发平台和工具链，提供托管的构建服务（EAS Build）、OTA 更新（EAS Update）、推送通知等，简化开发和部署流程。

**平台**：跨平台（React Native）

**关联术语**：React Native、Metro、EAS

---

## F

### Fabric (React Native New Architecture)

**定义**：React Native 新架构中的渲染系统，替代旧架构。核心组件包括 JSI、Turbo Modules、Fabric Renderer 和 Codegen，支持同步渲染和并发特性。

**平台**：跨平台（React Native）

**关联术语**：Bridge、JSI、Turbo Modules

### FlashList

**定义**：Shopify 开源的高性能列表组件，替代 React Native 的 FlatList。通过更好的单元格回收和布局策略显著提升长列表滚动性能。

**平台**：跨平台（React Native）

**关联术语**：FlatList、RecyclerView、UICollectionView

### Flutter

**定义**：Google 开源的跨平台 UI 框架，使用 Dart 语言，自带渲染引擎（Skia/Impeller），不依赖平台原生 UI 组件。UI 一致性高、性能接近原生。

**平台**：跨平台

**关联术语**：React Native、Dart、Platform Channel

---

## G

### Gradle

**定义**：Android 项目的构建工具，使用 Groovy 或 Kotlin DSL 定义构建脚本。管理依赖、编译、打包、签名和发布。

**平台**：Android

**关联术语**：CocoaPods、SPM、Maven

---

## H

### Hermes

**定义**：Meta 为 React Native 优化的 JavaScript 引擎。支持字节码预编译（HBC），启动时间减少 50%+，内存使用减少 30%+。

**平台**：跨平台（React Native）

**关联术语**：JavaScriptCore、V8、JS Bundle

### Hot Reload

**定义**：在不重启应用的情况下实时更新代码变更的开发特性。保留应用状态的同时更新 UI 和逻辑。Flutter 称 Hot Reload，React Native 称 Fast Refresh。

**关联术语**：Fast Refresh、CodePush

### Hybrid App

**定义**：使用 Web 技术（HTML/CSS/JavaScript）开发核心功能，通过 WebView 容器运行并调用原生能力的应用。代表框架：Ionic、Capacitor。

**关联术语**：WebView、PWA、Native App

---

## J

### Jank

**定义**：UI 渲染帧率低于目标刷新率（60fps = 16.67ms/帧）导致的视觉卡顿。常见原因：主线程耗时操作、过度绘制、布局嵌套过深。

**关联术语**：ANR、FPS、Main Thread

### JSI (JavaScript Interface)

**定义**：React Native 新架构中 JavaScript 与 C++ 层的直接通信接口，替代旧的异步 Bridge。允许 JS 同步调用原生方法，消除序列化开销。

**平台**：跨平台（React Native）

**关联术语**：Bridge、Turbo Modules、Fabric

---

## K

### Kotlin Multiplatform (KMP)

**定义**：JetBrains 的跨平台方案，允许在 iOS 和 Android 之间共享 Kotlin 业务逻辑代码，UI 层仍使用各平台原生框架（SwiftUI / Compose）。

**关联术语**：Flutter、React Native、Compose Multiplatform

---

## L

### Launch Screen / Splash Screen

**定义**：应用启动时显示的初始屏幕。iOS 使用 Storyboard 定义，Android 12+ 引入 SplashScreen API。应展示品牌标识，持续时间尽量短。

**关联术语**：Cold Start、App Startup

---

## M

### Mini Program

**定义**：运行在超级 App（微信、支付宝、抖音等）内部的轻量应用。使用类 Web 技术开发，在宿主 App 沙箱中运行。无需安装、即用即走，是中国市场特有的移动端形态。

**关联术语**：PWA、Hybrid App、WebView

### MMKV

**定义**：微信团队开源的高性能键值存储库，基于 mmap 内存映射，读写性能远超 SharedPreferences 和 AsyncStorage。

**平台**：Android / iOS / 跨平台

**关联术语**：AsyncStorage、SharedPreferences、UserDefaults

---

## N

### NDK (Native Development Kit)

**定义**：Android 原生开发工具包，允许使用 C/C++ 编写高性能代码模块。适用于计算密集型任务、游戏引擎和已有 C/C++ 库的集成。

**平台**：Android

**关联术语**：JNI、ABI、FFI

### Navigation

**定义**：应用内页面跳转和路由管理。各平台方案不同：iOS（NavigationStack）、Android（Navigation Component）、React Native（React Navigation / Expo Router）、Flutter（GoRouter）。

**关联术语**：Deep Link、Stack Navigation、Tab Navigation

---

## O

### OTA (Over-The-Air) Update

**定义**：通过网络推送应用更新，无需经过应用商店审核。React Native 使用 CodePush/Expo Updates，Flutter 使用 Shorebird。仅适用于 JS/Dart 层变更。

**关联术语**：CodePush、Expo Updates、Hot Reload

---

## P

### Platform Channel

**定义**：Flutter 中 Dart 代码与平台原生代码（Swift/Kotlin）之间的通信机制。支持 MethodChannel（方法调用）、EventChannel（事件流）和 BasicMessageChannel（消息传递）。

**平台**：Flutter

**关联术语**：Bridge、JSI、FFI

### ProGuard / R8

**定义**：Android 代码混淆和优化工具。R8 是 Google 的替代品（默认启用），功能包括代码缩减（Tree Shaking）、混淆和优化。

**平台**：Android

**关联术语**：DEX、APK、Bundle Size

### Push Notification

**定义**：服务器向移动设备推送消息的机制。iOS 使用 APNs，Android 使用 FCM。国内 Android 需适配各厂商推送通道（华为/小米/OPPO/vivo）。

**平台**：iOS / Android

**关联术语**：APNs、FCM、Local Notification

### PWA (Progressive Web App)

**定义**：使用 Web 技术构建的具备原生体验的 Web 应用。核心技术：Service Worker、Web App Manifest、Cache API。优势：无需应用商店、即时更新。劣势：iOS 支持有限。

**关联术语**：Hybrid App、Mini Program、WebView

---

## R

### React Native

**定义**：Meta 开源的跨平台移动框架，使用 React 和 JavaScript/TypeScript 构建原生应用。UI 组件映射到平台原生视图。新架构包含 JSI + Fabric + Turbo Modules。

**关联术语**：Flutter、Expo、Hermes、JSI

### Realm

**定义**：MongoDB 旗下的移动端数据库，面向对象的数据模型，支持实时同步和跨平台。比 SQLite 更易用。

**平台**：iOS / Android / 跨平台

**关联术语**：Core Data、Room、SQLite

### Retain Cycle

**定义**：iOS 中两个或多个对象相互强引用导致无法被 ARC 释放的内存泄漏。常见于闭包捕获 self、delegate 和定时器。使用 `weak` 或 `unowned` 修饰解决。

**平台**：iOS

**关联术语**：ARC、Memory Leak

### Room

**定义**：Android Jetpack 提供的 SQLite 抽象层，使用注解定义 Schema，编译时验证 SQL，支持 Flow 响应式数据。

**平台**：Android

**关联术语**：Core Data、Realm、SQLite

---

## S

### SPM (Swift Package Manager)

**定义**：Apple 官方的依赖管理工具，集成在 Xcode 中。逐步替代 CocoaPods 成为 iOS 生态主流方案。

**平台**：iOS / macOS

**关联术语**：CocoaPods、Gradle

### SwiftUI

**定义**：Apple 的声明式 UI 框架，使用 Swift 描述 UI，支持实时预览。iOS 13+ 可用，建议 iOS 15+ 使用以获得完整功能。

**平台**：iOS / macOS / watchOS / tvOS

**关联术语**：Compose、Flutter、UIKit

---

## T

### TestFlight

**定义**：Apple 官方的 Beta 测试分发平台。支持内部测试（25 人）和外部测试（10000 人），外部测试需通过审核。

**平台**：iOS

**关联术语**：App Signing、App Distribution

### Turbo Modules

**定义**：React Native 新架构中的原生模块系统，通过 JSI 实现同步调用，支持懒加载，由 Codegen 自动生成类型安全接口。

**平台**：跨平台（React Native）

**关联术语**：JSI、Bridge、Fabric

---

## U

### Universal Links / App Links

**定义**：平台级深度链接方案。Universal Links（iOS）和 App Links（Android）使用 HTTPS URL，系统根据关联配置决定打开 App 还是浏览器。

**平台**：iOS / Android

**关联术语**：Deep Link、URI Scheme

---

## W

### WebView

**定义**：在原生应用中嵌入的浏览器视图组件。iOS 使用 WKWebView，Android 使用基于 Chromium 的 WebView。性能低于原生渲染，应避免在核心页面大量使用。

**关联术语**：Hybrid App、PWA

### Widget

**定义**：有两个含义：(1) Flutter 中的 UI 构建块（一切皆 Widget）；(2) iOS/Android 桌面小组件（WidgetKit / App Widget），在主屏幕显示应用信息摘要。

**平台**：Flutter / iOS / Android

**关联术语**：Compose、SwiftUI、WidgetKit

---

## 术语对比速查表

| 维度 | iOS | Android | React Native | Flutter |
|------|-----|---------|-------------|---------|
| 语言 | Swift / ObjC | Kotlin / Java | JS / TS | Dart |
| UI 框架 | SwiftUI / UIKit | Compose / XML | RN Components | Flutter Widget |
| 包管理 | SPM / CocoaPods | Gradle / Maven | npm / yarn | pub |
| 本地存储 | Core Data / UserDefaults | Room / SharedPreferences | MMKV / AsyncStorage | Hive / sqflite |
| 推送 | APNs | FCM | Expo Notifications | Firebase Messaging |
| 深度链接 | Universal Links | App Links | React Navigation | GoRouter |
| 发布格式 | IPA | AAB / APK | -- | -- |
| 混淆工具 | -- | R8 / ProGuard | -- | obfuscate flag |
| Beta 测试 | TestFlight | Firebase App Distribution | -- | -- |

---

## Agent Checklist

以下为 AI Agent 在移动开发项目中使用本术语表的要点：

- [ ] 根据项目需求选择技术栈（原生 vs React Native vs Flutter vs KMP）
- [ ] React Native 项目确认是否启用新架构（JSI + Fabric + Turbo Modules）
- [ ] 确认 JS 引擎选择（React Native 推荐 Hermes）
- [ ] 列表组件使用高性能方案（FlashList / RecyclerView / UICollectionView）
- [ ] 本地存储根据场景选择（键值对用 MMKV，关系数据用 Room/Core Data）
- [ ] 深度链接使用平台级方案（Universal Links / App Links）而非仅 URI Scheme
- [ ] Android 发布使用 AAB 格式而非 APK
- [ ] iOS 注意 ARC 循环引用（闭包中使用 weak self）
- [ ] 跨平台项目注意原生模块的平台差异和桥接性能
- [ ] 建立性能监控：启动时间、帧率、内存、崩溃率、ANR 率
