---
id: i18n-internationalization-playbook
title: 国际化（i18n）实战手册
domain: frontend
category: 02-playbooks
difficulty: advanced
tags: [i18n, internationalization, l10n, localization, react-intl, next-intl, rtl, icu, translation, multilingual, enterprise]
quality_score: 93
maintainer: frontend-team@superdev.com
last_updated: 2026-06-15
---

# 国际化（i18n）实战手册

## 核心概念

- **i18n**（internationalization）：代码架构上支持多语言（不硬编码字符串）
- **l10n**（localization）：翻译适配到具体语言/地区
- **RTL**（Right-to-Left）：阿拉伯语/希伯来语从右到左的布局

## React/Next.js 实现

### next-intl（Next.js App Router 推荐）
```typescript
// messages/en.json
{ "HomePage": { "title": "Welcome", "greeting": "Hello, {name}!" } }
// messages/zh.json
{ "HomePage": { "title": "欢迎", "greeting": "你好，{name}！" } }

// 组件中使用
import { useTranslations } from 'next-intl';
function HomePage({ name }: { name: string }) {
  const t = useTranslations('HomePage');
  return (
    <>
      <h1>{t('title')}</h1>
      <p>{t('greeting', { name })}</p>  {/* ICU MessageFormat 插值 */}
    </>
  );
}
```

### react-intl（纯 React）
```tsx
import { FormattedMessage, IntlProvider } from 'react-intl';

<IntlProvider locale="zh" messages={zhMessages}>
  <FormattedMessage id="greeting" values={{ name: "张三" }} />
</IntlProvider>
```

## ICU MessageFormat（翻译标准）

```json
// 简单插值
"greeting": "Hello, {name}"

// 复数（不同语言复数规则不同！）
"items": "{count, plural, =0 {No items} one {# item} other {# items}}"
// 中文不区分单复数 → "其他" 分支
// 阿拉伯语有 6 种复数形式！

// 选择（性别等）
"pronoun": "{gender, select, male {he} female {she} other {they}}"

// 日期/数字（本地化格式）
"price": "{amount, number, ::currency USD}"
"date": "{date, date, ::yyyyMMdd}"
```

## RTL（从右到左）支持

```css
/* ✅ 用逻辑属性（不用 left/right） */
.card {
  margin-inline-start: 16px;   /* LTR: left, RTL: right */
  padding-inline-end: 24px;    /* LTR: right, RTL: left */
  text-align: start;           /* LTR: left, RTL: right */
  inset-inline-start: 0;       /* 替代 left: 0 */
}

/* ❌ 用物理属性（RTL 下布局错乱） */
.card {
  margin-left: 16px;   /* RTL 下应该是 right */
}

/* 方向切换 */
[dir="rtl"] .icon-arrow { transform: scaleX(-1); }  /* 箭头翻转 */
```

```tsx
// Next.js 设置方向
<html lang="ar" dir="rtl">
```

## 日期/时间/数字本地化

```typescript
// ✅ 用 Intl API（浏览器原生，无需库）
new Intl.DateTimeFormat('zh-CN', { dateStyle: 'full' })
  .format(new Date());  // "2024年6月15日星期六"

new Intl.NumberFormat('de-DE', { style: 'currency', currency: 'EUR' })
  .format(99.5);  // "99,50 €"

new Intl.NumberFormat('en-US').format(1000000);  // "1,000,000"
new Intl.NumberFormat('de-DE').format(1000000);  // "1.000.000"
```

## 生产检查清单
- [ ] 所有用户可见字符串走翻译文件（无硬编码）
- [ ] ICU MessageFormat 处理插值/复数/性别
- [ ] 日期/时间/数字用 Intl API（不用手写格式）
- [ ] RTL 布局用逻辑 CSS 属性（inline-start/end）
- [ ] 语言切换不刷新页面（动态加载 locale 消息）
- [ ] 翻译文件按页面/功能拆分（不一个巨型 JSON）
- [ ] 翻译 key 命名规范（`PageName.elementName`）
- [ ] 缺失翻译有 fallback（不显示 key 本身）
- [ ] SEO 的 hreflang 标签（多语言 URL）
- [ ] 输入法支持（IME composition 不打断）
