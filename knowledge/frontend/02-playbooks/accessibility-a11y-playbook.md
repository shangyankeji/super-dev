---
id: accessibility-a11y-playbook
title: Web 可访问性（a11y）实战手册
domain: frontend
category: 02-playbooks
difficulty: advanced
tags: [accessibility, a11y, wcag, aria, screen-reader, keyboard, color-contrast, semantic-html, ada, enterprise]
quality_score: 94
maintainer: frontend-team@superdev.com
last_updated: 2026-06-15
---

# Web 可访问性（a11y）实战手册

## WCAG 2.2 四大原则（POUR）

| 原则 | 含义 | 示例 |
|------|------|------|
| **可感知** (Perceivable) | 用户能感知所有信息 | 图片有 alt，视频有字幕 |
| **可操作** (Operable) | 用户能操作所有功能 | 键盘可导航，无时间限制 |
| **可理解** (Understandable) | 内容和操作可理解 | 简洁语言，错误提示清晰 |
| **健壮** (Robust) | 兼容辅助技术 | 语义 HTML，ARIA 正确 |

## 语义 HTML（最重要的 a11y）

```tsx
// ❌ 用 div 做所有事（屏幕阅读器无法理解）
<div class="button" onclick="save()">Save</div>
<div class="nav">
  <div class="link" onclick="goto('/home')">Home</div>
</div>

// ✅ 语义标签（屏幕阅读器自动理解角色）
<button onclick={save}>Save</button>
<nav>
  <a href="/home">Home</a>
</nav>
<main>          {/* 页面主内容 */}
  <article>     {/* 独立内容块 */}
    <h1>Title</h1>
    <section aria-labelledby="section-title">
      <h2 id="section-title">Section</h2>
    </section>
  </article>
</main>
<aside>         {/* 侧边栏 */}
<footer>        {/* 页脚 */}
```

## 键盘导航

```tsx
// ✅ 所有交互元素键盘可操作
// Tab 顺序合理（视觉顺序 = DOM 顺序）
// :focus-visible 样式可见
// 模态框：焦点陷阱（Tab 不跳出）+ Esc 关闭

function Modal({ onClose, children }) {
  const dialogRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    // 焦点陷阱
    const handleTab = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
      // ... Tab 循环逻辑
    };
    document.addEventListener('keydown', handleTab);
    dialogRef.current?.focus();  // 打开时聚焦
    return () => document.removeEventListener('keydown', handleTab);
  }, []);

  return (
    <div
      ref={dialogRef}
      role="dialog"
      aria-modal="true"
      aria-labelledby="modal-title"
      tabIndex={-1}   {/* 可聚焦但不进 Tab 序列 */}
    >
      <h2 id="modal-title">Title</h2>
      {children}
    </div>
  );
}
```

## ARIA 正确用法

```tsx
// ✅ ARIA 是补充，不是替代语义 HTML
// 规则 1：不要用 ARIA 改变元素的原生语义
<button role="tab">  ❌ 改用 <div role="tab" tabIndex={0}>

// ✅ 正确的 ARIA 模式
<div role="tablist">
  <button role="tab" aria-selected="true" aria-controls="panel-1" id="tab-1">Tab 1</button>
  <button role="tab" aria-selected="false" aria-controls="panel-2" id="tab-2">Tab 2</button>
</div>
<div role="tabpanel" id="panel-1" aria-labelledby="tab-1">Content 1</div>

// ✅ 动态内容用 aria-live 通知
<div aria-live="polite">{statusMessage}</div>     {/* 搜索结果数 */}
<div aria-live="assertive">{errorMessage}</div>    {/* 表单错误 */}
```

## 颜色对比度

| 元素 | 最小对比度（WCAG AA） |
|------|---------------------|
| 正文文本 | 4.5:1 |
| 大文本（18pt+） | 3:1 |
| UI 组件/图标 | 3:1 |

```css
/* ✅ 用 CSS 变量管理对比度 */
:root {
  --text-primary: #1a1a1a;    /* 对 #fff 对比度 15:1 ✓ */
  --text-secondary: #5a5a5a;  /* 对 #fff 对比度 5.7:1 ✓ */
  --text-muted: #767676;      /* 对 #fff 对比度 4.5:1 ✓（刚好 AA） */
  /* ❌ #999 对 #fff = 2.85:1（不达标） */
}
```

## 表单可访问性

```tsx
// ✅ label 关联（不用 placeholder 替代）
<label htmlFor="email">Email</label>
<input id="email" type="email" aria-describedby="email-hint" aria-invalid={!!error} />
<span id="email-hint">{error || "We'll never share your email"}</span>

// ✅ 错误提示关联
{error && <span id="email-error" role="alert">{error}</span>}
```

## 图片替代文本

```tsx
// 装饰图片（空 alt）
<img src="/decorative-line.png" alt="" />  {/* 屏幕阅读器跳过 */}

// 信息图片（描述性 alt）
<img src="/chart-q3.png" alt="Q3 revenue: $1.2M, up 15% from Q2" />

// 复杂图表（长描述）
<img src="/architecture.png" alt="System architecture" aria-describedby="arch-desc" />
<div id="arch-desc">The system consists of...（详细文字描述）</div>
```

## 生产检查清单
- [ ] 所有交互元素是语义 HTML（button/nav/main）
- [ ] 键盘可完整操作（Tab + Enter + Esc）
- [ ] :focus-visible 样式可见
- [ ] 颜色对比度 ≥ 4.5:1（正文）
- [ ] 图片有 alt（信息图片描述，装饰图片空 alt）
- [ ] 表单 label 正确关联
- [ ] 错误用 role="alert" + aria-live
- [ ] 模态框有焦点陷阱
- [ ] 页面有 skip-to-content 链接
- [ ] axe-core / Lighthouse a11y 审计 ≥ 90 分
- [ ] 屏幕阅读器实测（NVDA/VoiceOver）
