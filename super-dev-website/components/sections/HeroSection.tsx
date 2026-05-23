'use client';
import dynamic from 'next/dynamic';
import Link from 'next/link';
import { ArrowRight, BookOpen } from 'lucide-react';
import { Badge } from '@/components/ui/Badge';
import { CopyCommand } from '@/components/ui/CopyCommand';
import { formatStarCount, GITHUB_REPO_URL } from '@/lib/github';
import { useGithubStars } from '@/lib/useGithubStars';
import { localizedPath, type SiteLocale } from '@/lib/site-locale';

const TerminalWindow = dynamic(
  () => import('@/components/ui/TerminalWindow').then((mod) => mod.TerminalWindow),
  { ssr: false }
);

const COPY = {
  zh: {
    openSource: 'MIT 开源',
    title: 'AI 能写代码。Super Dev 是宿主的教练，让宿主流水线式交付完整商业项目。',
    body: 'Super Dev 本身不写代码、不是 IDE、不是 agent——它是 AI 编码宿主的教练，交给宿主一份完整的商业项目交付规范（SUPER_DEV_HOST_SPEC_V1）：先研究什么、产出哪些工件、何时暂停等确认、什么不许写、留下什么证据。宿主已有的模型 + 工具去执行；教练定的规范，决定了产物是不是商业级。',
    points: ['一份规范本体，9 个阶段、4 层强制约束（代码权重 / 流程契约 / 交付产物 / 证据链）', '同一份规范注入到 20+ 宿主原生面：Claude Code / Cursor / Codex / Cline / Windsurf / Droid / Kiro …', '装上即生效，零配置；产出可直接对接 SOC2 / ISO27001 合规证据链'],
    docs: '查看文档',
    installNote: '首页默认只讲 uv 安装和 super-dev 引导。安装器会直接告诉你推荐宿主、标准流第一句、比赛流第一句和接入后先验；终端到这里就该退场，日常开发回宿主里的 /super-dev、$super-dev 或 super-dev:。',
    releaseNote: 'v2.4.1: 当前修复重点已经切到 backend-only 交付口径、原生测试命令识别、轻量恢复上下文、文档生命周期索引，以及用户级接入面一致性。',
  },
  en: {
    openSource: 'MIT Open Source',
    title: 'AI can write code. Super Dev coaches your host through a standardized pipeline that ships complete commercial projects.',
    body: 'Super Dev is not a code generator, not an IDE, not an agent — it is a coach for AI coding hosts. It hands the host a complete commercial-project delivery specification (SUPER_DEV_HOST_SPEC_V1): what to research first, what artifacts to ship, when to pause for sign-off, what to refuse to write, what evidence to leave behind. The host’s existing model + tools execute; the coach’s standard is what makes the result commercial-grade.',
    points: ['One specification, 9 phases, 4 enforced layers (code rules / flow contract / artifacts / evidence chain)', 'The same spec is injected into 20+ host native surfaces: Claude Code / Cursor / Codex / Cline / Windsurf / Droid / Kiro …', 'Install once, zero config; outputs map directly to SOC 2 / ISO 27001 compliance evidence'],
    docs: 'Read Docs',
    installNote: 'The homepage now teaches uv install and the super-dev onboarding path. The installer prints the recommended host, the standard-flow first prompt, the competition-flow first prompt, and the post-onboard self-check. After that, the terminal should get out of the way and daily work moves back into /super-dev, $super-dev, or super-dev: inside the host.',
    releaseNote: 'v2.4.1 focuses on backend-only delivery closure, native test-command detection, lightweight resume context, document lifecycle indexing, and user-surface onboarding consistency.',
  },
} as const;

export function HeroSection({ locale = 'zh' }: { locale?: SiteLocale }) {
  const stars = useGithubStars();
  const copy = COPY[locale];

  return (
    <section className="relative overflow-hidden border-b border-border-muted bg-bg-primary pt-24 lg:pt-28" aria-labelledby="hero-title">
      <div className="absolute inset-0 pointer-events-none" aria-hidden="true">
        <div className="absolute left-1/2 top-0 h-[460px] w-[780px] -translate-x-1/2 rounded-full bg-accent-blue/6 blur-3xl" />
        <div className="absolute bottom-0 left-0 right-0 h-px bg-gradient-to-r from-transparent via-border-default to-transparent" />
      </div>

      <div className="relative mx-auto flex w-full max-w-7xl flex-col gap-14 px-4 pb-20 sm:px-6 lg:grid lg:grid-cols-[minmax(0,1fr)_520px] lg:items-center lg:gap-16 lg:pb-24">
        <div className="flex flex-col gap-7">
          <div className="flex flex-wrap items-center gap-2">
            <Badge variant="version">v2.4.1</Badge>
            <Badge variant="certified">{copy.openSource}</Badge>
            <a
              href={GITHUB_REPO_URL}
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center gap-1 text-sm text-text-muted transition-colors hover:text-text-secondary"
            >
              {formatStarCount(stars)} Stars
            </a>
          </div>

          <h1 id="hero-title" className="max-w-3xl text-4xl font-bold leading-[1.06] tracking-tight text-text-primary sm:text-5xl lg:text-6xl">
            {locale === 'zh' ? (
              <>
                AI 能写代码，
                <span className="text-gradient-brand">Super Dev</span>
                让项目能交付。
              </>
            ) : (
              <>
                AI can write code. <span className="text-gradient-brand">Super Dev</span> helps teams ship it properly.
              </>
            )}
          </h1>

          <p className="max-w-2xl text-lg leading-8 text-text-secondary">{copy.body}</p>

          <ul className="grid gap-3 text-sm text-text-secondary sm:grid-cols-3">
            {copy.points.map((point) => (
              <li key={point} className="rounded-xl border border-border-default bg-bg-secondary/55 px-4 py-3 leading-6">
                {point}
              </li>
            ))}
          </ul>

          <div id="get-started" className="flex flex-col gap-3 pt-1 sm:flex-row sm:flex-wrap sm:items-center">
            <CopyCommand command="uv tool install super-dev" className="sm:w-auto" />
            <Link
              href={localizedPath(locale, '/docs')}
              className="inline-flex items-center justify-center gap-2 rounded-lg border border-border-default px-4 py-3 text-sm font-medium text-text-secondary transition-all duration-150 hover:border-border-emphasis hover:text-text-primary"
            >
              <BookOpen size={16} aria-hidden="true" />
              {copy.docs}
              <ArrowRight size={14} aria-hidden="true" />
            </Link>
          </div>

          <div className="space-y-2 text-sm text-text-muted">
            <p>{copy.installNote}</p>
            <p>{copy.releaseNote}</p>
          </div>
        </div>

        <div className="relative">
          <TerminalWindow className="w-full" locale={locale} />
          <div className="pointer-events-none absolute -bottom-10 -right-8 h-48 w-48 rounded-full bg-accent-blue/10 blur-2xl" aria-hidden="true" />
        </div>
      </div>
    </section>
  );
}
