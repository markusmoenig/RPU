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
        <p className={styles.eyebrow}>Experimental, evolving language/runtime</p>
        <div className={styles.brandRow}>
          <h1 className={styles.title}>{siteConfig.title}</h1>
          <img className={styles.logo} src={logoUrl} alt="RPU logo" />
        </div>
        <p className={styles.tagline}>{siteConfig.tagline}</p>
        <p className={styles.copy}>
          RPU is a language and runtime for interactive 2D games and apps, with declarative
          scenes, lightweight scripting, and a CLI-first workflow.
        </p>
        <div className={styles.actions}>
          <Link className="button button--primary button--lg" to="/intro">
            Read the docs
          </Link>
          <Link className="button button--secondary button--lg" to="/examples">
            See examples
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
      <HomePageHeader />
    </Layout>
  );
}
