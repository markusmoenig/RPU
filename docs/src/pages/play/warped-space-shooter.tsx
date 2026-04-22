import type {ReactNode} from 'react';
import Layout from '@theme/Layout';

export default function WarpedSpaceShooterPlay(): ReactNode {
  return (
    <Layout
      title="Play Warped Space Shooter"
      description="Play the RPU web build of Warped Space Shooter.">
      <main
        style={{
          minHeight: 'calc(100vh - 60px)',
          background: '#050b18',
          padding: '0',
          margin: '0',
        }}>
        <iframe
          src="/play/warped-space-shooter/index.html"
          title="Warped Space Shooter"
          style={{
            display: 'block',
            width: '100%',
            height: 'calc(100vh - 60px)',
            border: '0',
            background: '#050b18',
          }}
        />
      </main>
    </Layout>
  );
}
