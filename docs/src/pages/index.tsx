import type {ReactNode} from 'react';
import Link from '@docusaurus/Link';
import useBaseUrl from '@docusaurus/useBaseUrl';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';

import styles from './index.module.css';

function HomePageHeader() {
  const {siteConfig} = useDocusaurusContext();
  const logoUrl = useBaseUrl('/img/logo-w.png');

  return (
    <header className={styles.hero}>
      <div className={styles.inner}>
        <p className={styles.eyebrow}>Fantasy Computer</p>
        <div className={styles.heroRow}>
          <div className={styles.heroText}>
            <h1 className={styles.title}>{siteConfig.title}</h1>
            <p className={styles.tagline}>{siteConfig.tagline}</p>
            <p className={styles.copy}>
              RPU is a tiny creative computer for tools, apps, games, and creative modules. App
              cartridges can open a window and run scenes, CLI cartridges can run headless tools,
              and future modules can come from RPU bytecode, WASM, or trusted native backends.
            </p>
            <div className={styles.actions}>
              <Link className="button button--primary button--lg" to="/getting-started">
                Getting Started
              </Link>
              <Link className="button button--secondary button--lg" to="/concept">
                Read Concept
              </Link>
              <Link className="button button--secondary button--lg" to="/examples">
                See Examples
              </Link>
            </div>
          </div>
          <img className={styles.heroLogo} src={logoUrl} alt="RPU logo" />
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
      description="RPU fantasy computer and cartridge runtime documentation">
      <div className={styles.pageShell}>
        <HomePageHeader />
        <main className={styles.main}>
          <div className={styles.content}>
            <section className={styles.section}>
              <h2>Run a Cartridge</h2>
              <p>
                A cartridge declares what kind of software it is and which tiny-computer services
                it needs from the host.
              </p>
              <p>Start with a compact manifest:</p>
              <div className={styles.codeBlock}>
                <pre className={styles.codePre}>
                  <code className={styles.code}>
                    <span className={styles.codeLine}>
                      <span className={styles.pn}>[</span><span className={styles.kw}>project</span><span className={styles.pn}>]</span>
                    </span>
                    <span className={styles.codeLine}>
                      name <span className={styles.op}>=</span> <span className={styles.str}>"hello_cli"</span>
                    </span>
                    <span className={styles.codeLine}>
                      kind <span className={styles.op}>=</span> <span className={styles.str}>"cli"</span>
                    </span>
                    <span className={styles.codeLine}>
                      {' '}
                    </span>
                    <span className={styles.codeLine}>
                      <span className={styles.pn}>[</span><span className={styles.kw}>build</span><span className={styles.pn}>]</span>
                    </span>
                    <span className={styles.codeLine}>
                      language <span className={styles.op}>=</span> <span className={styles.str}>"rpu"</span>
                    </span>
                    <span className={styles.codeLine}>
                      backend <span className={styles.op}>=</span> <span className={styles.str}>"bytecode"</span>
                    </span>
                    <span className={styles.codeLine}>
                      {' '}
                    </span>
                    <span className={styles.codeLine}>
                      <span className={styles.pn}>[</span><span className={styles.kw}>requires</span><span className={styles.pn}>]</span>
                    </span>
                    <span className={styles.codeLine}>
                      system <span className={styles.op}>=</span> <span className={styles.num}>true</span>
                    </span>
                    <span className={styles.codeLine}>
                      graphics <span className={styles.op}>=</span> <span className={styles.num}>false</span>
                    </span>
                    <span className={styles.codeLine}>
                      audio <span className={styles.op}>=</span> <span className={styles.num}>false</span>
                    </span>
                    <span className={styles.codeLine}>
                      network <span className={styles.op}>=</span> <span className={styles.num}>false</span>
                    </span>
                  </code>
                </pre>
              </div>
              <ol>
                <li><code>kind = "app"</code> opens a window and runs scenes.</li>
                <li><code>kind = "cli"</code> runs headless <code>on run()</code> scripts.</li>
                <li><code>build</code> selects the frontend and execution backend.</li>
                <li><code>requires</code> declares the runtime service families.</li>
              </ol>
            </section>

            <section className={styles.section}>
              <h2>Use the RPU DSL</h2>
              <p>
                The RPU DSL is the friendly built-in frontend. It can describe app scenes or small
                headless CLI tools while the runtime stays language-independent.
              </p>
              <div className={styles.codeBlock}>
                <pre className={styles.codePre}>
                  <code className={styles.code}>
                    <span className={styles.codeLine}>
                      <span className={styles.kw}>on</span> <span className={styles.fn}>run</span><span className={styles.pn}>() {'{'}</span>
                    </span>
                    <span className={styles.codeLine}>
                      {'    '}<span className={styles.fn}>print</span><span className={styles.pn}>(</span><span className={styles.str}>"Hello from CLI"</span><span className={styles.pn}>)</span>
                    </span>
                    <span className={styles.codeLine}>
                      {'    '}<span className={styles.kw}>if</span> <span className={styles.fn}>arg_count</span><span className={styles.pn}>()</span> <span className={styles.op}>{'>'}</span> <span className={styles.num}>0</span> <span className={styles.pn}>{'{'}</span>
                    </span>
                    <span className={styles.codeLine}>
                      {'        '}<span className={styles.fn}>print</span><span className={styles.pn}>(</span><span className={styles.str}>"first arg: "</span> <span className={styles.op}>+</span> <span className={styles.fn}>arg</span><span className={styles.pn}>(</span><span className={styles.num}>0</span><span className={styles.pn}>))</span>
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
                RPU bytecode gives the DSL fast iteration while bootstrapping. WASM should become
                the shared ABI for RPU, C, Rust, Zig, Odin, Denrim, and other frontends.
              </p>
            </section>
          </div>
        </main>
      </div>
    </Layout>
  );
}
