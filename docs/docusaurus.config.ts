import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';
import rehypeTreeSitterRpu from './src/rehype/rehypeTreeSitterRpu';

const config: Config = {
  title: 'RPU',
  tagline: 'The Game Language. Build. Run. Everywhere.',
  favicon: 'img/favicon.png',
  future: {
    v4: true,
  },
  url: 'https://rpu-lang.org',
  baseUrl: '/',
  onBrokenLinks: 'throw',
  markdown: {
    hooks: {
      onBrokenMarkdownLinks: 'throw',
    },
  },
  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },
  presets: [
    [
      'classic',
      {
        docs: {
          sidebarPath: './sidebars.ts',
          routeBasePath: '/',
          editUrl: 'https://github.com/markusmoenig/RPU/tree/main/docs/',
          rehypePlugins: [rehypeTreeSitterRpu],
        },
        blog: false,
        theme: {
          customCss: './src/css/custom.css',
        },
      } satisfies Preset.Options,
    ],
  ],
  themeConfig: {
    image: 'img/logo-w.png',
    colorMode: {
      respectPrefersColorScheme: true,
    },
    navbar: {
      title: 'RPU',
      logo: {
        alt: 'RPU logo',
        src: 'img/logo-w.png',
        href: '/',
      },
      items: [
        {to: '/getting-started', label: 'Docs', position: 'left'},
        {to: '/examples', label: 'Examples', position: 'left'},
        {
          type: 'html',
          position: 'right',
          value: `
            <a href="https://github.com/markusmoenig/RPU" class="navbar-icon" title="GitHub Repository">
              <img src="https://img.shields.io/github/stars/markusmoenig/RPU?style=flat&color=d64d33&logo=github" alt="GitHub stars"/>
            </a>
          `,
        },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Social',
          items: [
            {
              html: '<a class="footer__link-item social-link" href="https://x.com/MarkusMoenig" target="_blank" rel="noopener noreferrer"><span class="social-icon">𝕏</span><span>X</span></a>',
            },
            {
              html: '<a class="footer__link-item social-link" href="https://bsky.app/profile/markusmoenig.bsky.social" target="_blank" rel="noopener noreferrer"><span class="social-icon">☁</span><span>Bluesky</span></a>',
            },
            {
              html: '<a class="footer__link-item social-link" href="https://github.com/markusmoenig/RPU" target="_blank" rel="noopener noreferrer"><span class="social-icon">⌘</span><span>GitHub Repo</span></a>',
            },
          ],
        },
        {
          title: 'Docs',
          items: [
            {label: 'Getting Started', to: '/getting-started'},
            {label: 'Scenes', to: '/scenes'},
            {label: 'Scripts', to: '/scripts'},
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} Markus Moenig.`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
      additionalLanguages: ['toml', 'rust'],
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
