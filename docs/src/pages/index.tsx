import type {ReactNode} from 'react';
import Link from '@docusaurus/Link';
import useBaseUrl from '@docusaurus/useBaseUrl';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';

import styles from './index.module.css';

function HomePageHeader() {
  const {siteConfig} = useDocusaurusContext();
  const logoUrl = useBaseUrl('/img/logo.png');

  return (
    <header className={styles.hero}>
      <div className={styles.inner}>
        <p className={styles.eyebrow}>Scene DSL + Script Runtime</p>
        <div className={styles.brandRow}>
          <h1 className={styles.title}>{siteConfig.title}</h1>
          <img className={styles.logo} src={logoUrl} alt="RPU logo" />
        </div>
        <p className={styles.tagline}>{siteConfig.tagline}</p>
        <p className={styles.copy}>
          RPU is a language and runtime for interactive 2D games and apps, with declarative
          scenes, lightweight scripting, hot reload, and a CLI-based workflow. Build for every
          platform through the power of Rust.
        </p>
        <div className={styles.actions}>
          <Link className="button button--primary button--lg" to="/getting-started">
            Getting Started
          </Link>
          <Link className="button button--secondary button--lg" to="/examples">
            See Examples
          </Link>
        </div>
      </div>
    </header>
  );
}

export default function Home(): ReactNode {
  const {siteConfig} = useDocusaurusContext();

  return (
    <Layout
      title={siteConfig.title}
      description="RPU language and runtime documentation">
      <div className={styles.pageShell}>
        <HomePageHeader />
        <main className={styles.main}>
          <div className={styles.content}>
            <section className={styles.section}>
              <h2>Build a Scene</h2>
              <p>
                RPU combines declarative scenes with lightweight scripting. You describe what is
                on screen, attach behavior where it belongs, and run it through the CLI.
              </p>
              <p>Start with a single sprite:</p>
              <div className={styles.codeBlock}>
                <pre className={styles.codePre}>
                  <code className={styles.code}>
                    <span className={styles.codeLine}>
                      <span className={styles.kw}>scene</span> Main <span className={styles.pn}>{'{'}</span>
                    </span>
                    <span className={styles.codeLine}>
                      {'    '}
                      <span className={styles.kw}>sprite</span> Hero <span className={styles.pn}>{'{'}</span>
                    </span>
                    <span className={styles.codeLine}>
                      {'        '}pos <span className={styles.op}>=</span> <span className={styles.pn}>(</span>
                      <span className={styles.num}>48</span>, <span className={styles.num}>56</span>
                      <span className={styles.pn}>)</span>
                    </span>
                    <span className={styles.codeLine}>
                      {'        '}texture <span className={styles.op}>=</span>{' '}
                      <span className={styles.str}>"hero.png"</span>
                    </span>
                    <span className={styles.codeLine}>
                      {'        '}color <span className={styles.op}>=</span>{' '}
                      <span className={styles.col}>#f4f8ff</span>
                    </span>
                    <span className={styles.codeLine}>
                      {'    '}<span className={styles.pn}>{'}'}</span>
                    </span>
                    <span className={styles.codeLine}>
                      <span className={styles.pn}>{'}'}</span>
                    </span>
                  </code>
                </pre>
              </div>
              <ol>
                <li>Creates a scene named <code>Main</code>.</li>
                <li>Places a sprite called <code>Hero</code>.</li>
                <li>Draws <code>hero.png</code> with a starting position and tint.</li>
              </ol>
            </section>

            <section className={styles.section}>
              <h2>Add Behavior</h2>
              <p>
                Then attach a script directly to the same node. The scene still owns structure.
                The script only owns behavior.
              </p>
              <div className={styles.codeBlock}>
                <pre className={styles.codePre}>
                  <code className={styles.code}>
                    <span className={styles.codeLine}>
                      <span className={styles.kw}>scene</span> Main <span className={styles.pn}>{'{'}</span>
                    </span>
                    <span className={styles.codeLine}>
                      {'    '}
                      <span className={styles.kw}>sprite</span> Hero <span className={styles.pn}>{'{'}</span>
                    </span>
                    <span className={styles.codeLine}>
                      {'        '}pos <span className={styles.op}>=</span> <span className={styles.pn}>(</span>
                      <span className={styles.num}>48</span>, <span className={styles.num}>56</span>
                      <span className={styles.pn}>)</span>
                    </span>
                    <span className={styles.codeLine}>
                      {'        '}texture <span className={styles.op}>=</span>{' '}
                      <span className={styles.str}>"hero.png"</span>
                    </span>
                    <span className={styles.codeLine}>
                      {'        '}
                    </span>
                    <span className={styles.codeLine}>
                      {'        '}<span className={styles.kw}>on</span>{' '}
                      <span className={styles.fn}>update</span>
                      <span className={styles.pn}>(</span>
                      <span className={styles.param}>dt</span>
                      <span className={styles.pn}>) {'{'}</span>
                    </span>
                    <span className={styles.codeLine}>
                      {'            '}<span className={styles.kw}>if</span>{' '}
                      <span className={styles.fn}>input_left</span>
                      <span className={styles.pn}>()</span> <span className={styles.pn}>{'{'}</span>
                    </span>
                    <span className={styles.codeLine}>
                      {'                '}self<span className={styles.pn}>.</span>x{' '}
                      <span className={styles.op}>=</span> self<span className={styles.pn}>.</span>x{' '}
                      <span className={styles.op}>-</span> <span className={styles.num}>120.0</span>{' '}
                      <span className={styles.op}>*</span> dt
                    </span>
                    <span className={styles.codeLine}>
                      {'            '}<span className={styles.pn}>{'}'}</span>
                    </span>
                    <span className={styles.codeLine}>
                      {'        '}<span className={styles.pn}>{'}'}</span>
                    </span>
                    <span className={styles.codeLine}>
                      {'    '}<span className={styles.pn}>{'}'}</span>
                    </span>
                    <span className={styles.codeLine}>
                      <span className={styles.pn}>{'}'}</span>
                    </span>
                  </code>
                </pre>
              </div>
              <p>
                That is the core RPU model: scene files describe what exists, scripts describe
                what changes, and the CLI builds and runs the project.
              </p>
            </section>
          </div>
        </main>
      </div>
    </Layout>
  );
}
