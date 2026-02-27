import React from 'react';
import clsx from 'clsx';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';
import styles from './index.module.css';

function Hero() {
  return (
    <header className={clsx('hero hero--primary', styles.hero)}>
      <div className="container">
        <h1 className="hero__title">ContextGenOS</h1>
        <p className="hero__subtitle">
          Your personal AI memory layer. Local. Private. Yours.
        </p>
        <p className={styles.tagline}>
          Collect context from your calendar, notes, and browser history.
          Store it locally with privacy controls. Inject it into any AI tool via MCP.
        </p>
        <div className={styles.buttons}>
          <Link className="button button--secondary button--lg" to="/docs/quickstart">
            Get Started — 10 min ⏱
          </Link>
          <Link
            className="button button--outline button--secondary button--lg"
            href="https://github.com/Kisbjornssund/ContextGenOS"
          >
            GitHub
          </Link>
        </div>
      </div>
    </header>
  );
}

const features = [
  {
    title: 'Local-first',
    description:
      'Everything stays on your machine. No cloud sync, no third-party storage. Your context data never leaves your device unless you choose to inject it.',
  },
  {
    title: 'Privacy controls',
    description:
      'Sensitivity levels and per-model rules. Mark items personal, work, or confidential. Decide exactly what each AI tool can see.',
  },
  {
    title: 'MCP integration',
    description:
      'Works with Claude, Cursor, Zed, and any MCP-compatible tool. One server, all your AI tools — no copy-pasting context ever again.',
  },
  {
    title: 'Pluggable collectors',
    description:
      'Python SDK to build collectors for any data source. Calendar, notes, browser history, and more — ship your own in an afternoon.',
  },
  {
    title: 'Cross-platform',
    description:
      'Runs on macOS, Linux, and Windows. Rust core for performance and safety. Docker image available for headless deployments.',
  },
  {
    title: 'Open source',
    description:
      'MIT licensed. Audit the code, fork it, self-host it. No telemetry, no accounts, no lock-in.',
  },
];

function Features() {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className="row">
          {features.map(({ title, description }) => (
            <div key={title} className={clsx('col col--4', styles.featureCol)}>
              <h3>{title}</h3>
              <p>{description}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

export default function Home() {
  const { siteConfig } = useDocusaurusContext();
  return (
    <Layout title={siteConfig.title} description={siteConfig.tagline}>
      <Hero />
      <main>
        <Features />
        <section className={styles.cta}>
          <div className="container">
            <h2>Ready to get started?</h2>
            <p>Install in one command, connect to Claude in under 10 minutes.</p>
            <pre className={styles.installCmd}>
              curl -fsSL https://contextgenos.dev/install.sh | sh
            </pre>
            <Link className="button button--primary button--lg" to="/docs/quickstart">
              Read the Quickstart
            </Link>
          </div>
        </section>
      </main>
    </Layout>
  );
}
