import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

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
        },
        blog: false,
        theme: {
          customCss: './src/css/custom.css',
        },
      } satisfies Preset.Options,
    ],
  ],
  themeConfig: {
    image: 'img/logo.png',
    colorMode: {
      respectPrefersColorScheme: true,
    },
    navbar: {
      title: 'RPU',
      logo: {
        alt: 'RPU logo',
        src: 'img/logo.png',
        href: '/',
      },
      items: [
        {to: '/intro', label: 'Docs', position: 'left'},
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
          title: 'Docs',
          items: [
            {label: 'Overview', to: '/intro'},
            {label: 'Scripts', to: '/scripts'},
          ],
        },
        {
          title: 'Project',
          items: [
            {label: 'rpu-lang.org', href: 'https://rpu-lang.org'},
            {label: 'GitHub', href: 'https://github.com/markusmoenig/RPU'},
            {label: 'Examples', to: '/examples'},
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
