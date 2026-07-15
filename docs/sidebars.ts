import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

const sidebars: SidebarsConfig = {
  tutorialSidebar: [
    'getting-started',
    'concept',
    'cartridges',
    'runtime-services',
    'system-api',
    'gpu',
    'modules',
    'wasm-abi',
    {
      type: 'category',
      label: 'SDKs',
      items: ['sdks', 'c-sdk'],
    },
    'cli',
    {
      type: 'category',
      label: 'RPU DSL',
      items: ['rpu-dsl', 'scenes', 'scripts'],
    },
    'examples',
  ],
};

export default sidebars;
