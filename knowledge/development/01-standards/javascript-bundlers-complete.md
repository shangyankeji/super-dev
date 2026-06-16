---
id: javascript-bundlers-complete
title: JavaScript打包工具完整指南
domain: development
category: 01-standards
difficulty: intermediate
tags: [agent, bundlers, checklist, complete, development, javascript, 实战代码示例, 常见陷阱]
quality_score: 70
last_updated: 2026-06-15
---
# JavaScript打包工具完整指南

## 概述
JavaScript打包工具将模块化的源代码转换为可在浏览器或Node.js中运行的优化产物。本指南深入对比Vite、Webpack、Rollup、esbuild、Turbopack五大工具,覆盖配置、优化策略和迁移方案。

## 核心概念

### 1. 打包工具的作用
- **模块解析**: 处理import/require依赖关系
- **代码转换**: TypeScript/JSX/CSS预处理器编译
- **Tree Shaking**: 消除未使用的代码
- **代码分割**: 按需加载,减少初始加载体积
- **资产处理**: 图片/字体/SVG等静态资源
- **开发体验**: HMR(热模块替换)、Source Map

### 2. 工具对比总览

| 特性 | Vite | Webpack | Rollup | esbuild | Turbopack |
|------|------|---------|--------|---------|-----------|
| 语言 | JS(基于Rollup+esbuild) | JS | JS | Go | Rust |
| 开发服务器 | 原生ESM,极快 | Bundle后服务 | 无内置 | 有限 | 增量编译 |
| HMR速度 | ~50ms | ~500ms-2s | 无内置 | 无内置 | ~10ms |
| 生产构建 | Rollup | 自身 | 自身 | 自身 | 开发中 |
| 配置复杂度 | 低 | 高 | 中 | 低 | 低 |
| 插件生态 | 丰富(兼容Rollup) | 最丰富 | 丰富 | 有限 | 发展中 |
| 适用场景 | 现代Web应用 | 企业级复杂项目 | 库/SDK | 极速构建 | Next.js |

### 3. 构建模式
- **开发模式**: 优先速度,Source Map完整,不压缩
- **生产模式**: 优先体积,Tree Shaking,压缩混淆
- **SSR模式**: 服务端渲染产物,Node.js兼容

## 实战代码示例

### Vite配置

```typescript
// vite.config.ts
import { defineConfig, splitVendorChunkPlugin } from 'vite'
import react from '@vitejs/plugin-react'
import { visualizer } from 'rollup-plugin-visualizer'

export default defineConfig(({ mode }) => ({
  plugins: [
    react(),
    splitVendorChunkPlugin(),
    mode === 'analyze' && visualizer({
      open: true,
      gzipSize: true,
    }),
  ].filter(Boolean),

  resolve: {
    alias: {
      '@': '/src',
      '@components': '/src/components',
      '@utils': '/src/utils',
    },
  },

  server: {
    port: 3000,
    proxy: {
      '/api': {
        target: 'http://localhost:8080',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api/, ''),
      },
    },
  },

  build: {
    target: 'es2020',
    outDir: 'dist',
    sourcemap: true,
    rollupOptions: {
      output: {
        manualChunks: {
          'vendor-react': ['react', 'react-dom', 'react-router-dom'],
          'vendor-ui': ['@radix-ui/react-dialog', '@radix-ui/react-dropdown-menu'],
          'vendor-utils': ['date-fns', 'lodash-es'],
        },
      },
    },
    chunkSizeWarningLimit: 500,
  },

  css: {
    modules: {
      localsConvention: 'camelCase',
    },
    preprocessorOptions: {
      scss: {
        additionalData: `@import "@/styles/variables.scss";`,
      },
    },
  },

  optimizeDeps: {
    include: ['react', 'react-dom'],
  },
}))
```

### Webpack 5配置

```javascript
// webpack.config.js
const path = require('path')
const HtmlWebpackPlugin = require('html-webpack-plugin')
const MiniCssExtractPlugin = require('mini-css-extract-plugin')
const CssMinimizerPlugin = require('css-minimizer-webpack-plugin')
const TerserPlugin = require('terser-webpack-plugin')
const { BundleAnalyzerPlugin } = require('webpack-bundle-analyzer')

const isProd = process.env.NODE_ENV === 'production'

module.exports = {
  mode: isProd ? 'production' : 'development',
  entry: './src/index.tsx',
  output: {
    path: path.resolve(__dirname, 'dist'),
    filename: isProd ? '[name].[contenthash:8].js' : '[name].js',
    chunkFilename: isProd ? '[name].[contenthash:8].chunk.js' : '[name].chunk.js',
    clean: true,
  },

  resolve: {
    extensions: ['.ts', '.tsx', '.js', '.jsx'],
    alias: {
      '@': path.resolve(__dirname, 'src'),
    },
  },

  module: {
    rules: [
      {
        test: /\.tsx?$/,
        use: 'ts-loader',
        exclude: /node_modules/,
      },
      {
        test: /\.css$/,
        use: [
          isProd ? MiniCssExtractPlugin.loader : 'style-loader',
          'css-loader',
          'postcss-loader',
        ],
      },
      {
        test: /\.(png|jpg|gif|svg)$/,
        type: 'asset',
        parser: {
          dataUrlCondition: { maxSize: 8 * 1024 },
        },
      },
    ],
  },

  optimization: {
    minimizer: [
      new TerserPlugin({
        terserOptions: {
          compress: { drop_console: isProd },
        },
      }),
      new CssMinimizerPlugin(),
    ],
    splitChunks: {
      chunks: 'all',
      cacheGroups: {
        vendor: {
          test: /[\\/]node_modules[\\/]/,
          name: 'vendor',
          chunks: 'all',
          priority: 10,
        },
        common: {
          minChunks: 2,
          priority: 5,
          reuseExistingChunk: true,
        },
      },
    },
    runtimeChunk: 'single',
  },

  plugins: [
    new HtmlWebpackPlugin({
      template: './public/index.html',
      minify: isProd,
    }),
    isProd && new MiniCssExtractPlugin({
      filename: '[name].[contenthash:8].css',
    }),
    process.env.ANALYZE && new BundleAnalyzerPlugin(),
  ].filter(Boolean),

  devServer: {
    port: 3000,
    hot: true,
    historyApiFallback: true,
    proxy: [{
      context: ['/api'],
      target: 'http://localhost:8080',
    }],
  },

  devtool: isProd ? 'source-map' : 'eval-cheap-module-source-map',
}
```

### Rollup配置(库打包)

```javascript
// rollup.config.mjs
import resolve from '@rollup/plugin-node-resolve'
import commonjs from '@rollup/plugin-commonjs'
import typescript from '@rollup/plugin-typescript'
import terser from '@rollup/plugin-terser'
import dts from 'rollup-plugin-dts'
import { readFileSync } from 'fs'

const pkg = JSON.parse(readFileSync('./package.json', 'utf-8'))
const external = [
  ...Object.keys(pkg.dependencies || {}),
  ...Object.keys(pkg.peerDependencies || {}),
]

export default [
  // ESM + CJS 产物
  {
    input: 'src/index.ts',
    output: [
      {
        file: pkg.main,       // dist/index.cjs
        format: 'cjs',
        sourcemap: true,
        exports: 'named',
      },
      {
        file: pkg.module,     // dist/index.mjs
        format: 'esm',
        sourcemap: true,
      },
    ],
    external,
    plugins: [
      resolve(),
      commonjs(),
      typescript({ tsconfig: './tsconfig.json' }),
      terser(),
    ],
  },
  // 类型声明
  {
    input: 'src/index.ts',
    output: { file: 'dist/index.d.ts', format: 'esm' },
    plugins: [dts()],
  },
]
```

### esbuild配置

```javascript
// build.mjs
import { build, context } from 'esbuild'

const shared = {
  entryPoints: ['src/index.tsx'],
  bundle: true,
  external: ['react', 'react-dom'],
  target: ['es2020'],
  loader: {
    '.png': 'file',
    '.svg': 'dataurl',
  },
}

// 生产构建
await build({
  ...shared,
  outdir: 'dist',
  format: 'esm',
  splitting: true,
  minify: true,
  sourcemap: true,
  metafile: true,
  define: {
    'process.env.NODE_ENV': '"production"',
  },
})

// 开发服务器
const ctx = await context({
  ...shared,
  outdir: 'dist',
  format: 'esm',
  sourcemap: 'inline',
  define: {
    'process.env.NODE_ENV': '"development"',
  },
})

await ctx.serve({ port: 3000, servedir: 'dist' })
```

### package.json库发布配置

```json
{
  "name": "my-library",
  "version": "1.0.0",
  "type": "module",
  "main": "./dist/index.cjs",
  "module": "./dist/index.mjs",
  "types": "./dist/index.d.ts",
  "exports": {
    ".": {
      "import": {
        "types": "./dist/index.d.ts",
        "default": "./dist/index.mjs"
      },
      "require": {
        "types": "./dist/index.d.cts",
        "default": "./dist/index.cjs"
      }
    },
    "./utils": {
      "import": "./dist/utils.mjs",
      "require": "./dist/utils.cjs"
    }
  },
  "files": ["dist"],
  "sideEffects": false
}
```

## 最佳实践

### 1. 选型原则
- **新项目SPA/SSR**: Vite(开发快、配置少、生态好)
- **复杂企业应用**: Webpack(功能全、社区大、定制强)
- **NPM库/SDK**: Rollup(Tree Shaking好、产物干净)
- **工具链底层**: esbuild(极速、适合开发工具)
- **Next.js项目**: Turbopack(原生集成、增量编译)

### 2. 性能优化清单
- 配置代码分割(路由级/组件级懒加载)
- 合理设置manualChunks分离大型依赖
- 启用gzip/brotli压缩
- 图片使用现代格式(WebP/AVIF)
- 使用import()动态导入非关键模块
- 分析bundle组成(visualizer/analyzer)

### 3. 开发体验优化
- 使用resolve.alias简化导入路径
- 配置proxy避免CORS问题
- 启用HMR保持开发状态
- Source Map选择合适精度(开发eval,生产source-map)

### 4. 缓存策略
- 生产构建使用contenthash文件名
- 分离vendor chunk(依赖变化少)
- 配置runtimeChunk单独提取运行时
- CDN配置长缓存+指纹更新

## 常见陷阱

### 陷阱1: Tree Shaking失效
```javascript
// 错误: CommonJS导入无法Tree Shake
const _ = require('lodash')
_.get(obj, 'path')

// 正确: ESM导入支持Tree Shaking
import { get } from 'lodash-es'
get(obj, 'path')

// 确保package.json标记
{ "sideEffects": false }
// 或指定有副作用的文件
{ "sideEffects": ["*.css", "./src/polyfills.js"] }
```

### 陷阱2: 循环依赖
```javascript
// a.js
import { b } from './b.js'
export const a = () => b()

// b.js
import { a } from './a.js'  // 循环!可能得到undefined
export const b = () => a()

// 解决: 提取共享逻辑到第三个模块
```

### 陷阱3: 动态导入路径
```javascript
// 错误: 完全动态路径,打包工具无法分析
const module = await import(path)

// 正确: 使用模板字面量限制范围
const module = await import(`./pages/${name}.tsx`)
```

### 陷阱4: CSS顺序不确定
```javascript
// 错误: 多个CSS文件导入顺序可能变化
import './a.css'
import './b.css'  // 生产环境可能和开发环境顺序不同

// 正确: 使用CSS Modules或CSS-in-JS避免全局冲突
import styles from './component.module.css'
```

### 陷阱5: 开发生产环境不一致
```javascript
// Vite开发用原生ESM,生产用Rollup打包
// 确保依赖在optimizeDeps中预构建
optimizeDeps: {
  include: ['problematic-cjs-package'],
}
// 使用preview命令验证生产构建
// npm run build && npm run preview
```

## Agent Checklist

### 工具选型
- [ ] 根据项目类型选择合适的打包工具
- [ ] 评估团队已有经验和迁移成本
- [ ] 确认所需功能(SSR/WebWorker/WASM)支持

### 构建配置
- [ ] 开发/生产环境配置分离
- [ ] Source Map策略合理
- [ ] 代码分割配置到位
- [ ] 资源哈希确保缓存有效性

### 性能优化
- [ ] 分析bundle体积并优化大依赖
- [ ] Tree Shaking正常工作(ESM导入)
- [ ] 关键路径无不必要的动态导入
- [ ] gzip/brotli压缩已启用

### 开发体验
- [ ] HMR正常工作
- [ ] 构建速度在可接受范围
- [ ] 路径别名已配置
- [ ] 代理配置正确

### 生产就绪
- [ ] 生产构建无console.log
- [ ] CSS已提取为独立文件
- [ ] 图片/字体等资源已优化
- [ ] 环境变量未泄漏到客户端
