// @ts-check

/** @type {import('@docusaurus/types').Config} */
const config = {
  title: 'ContextGenOS',
  tagline: 'Your personal AI memory layer. Local. Private. Yours.',
  favicon: 'img/favicon.ico',

  url: 'https://docs.contextgenos.dev',
  baseUrl: '/',

  organizationName: 'Kisbjornssund',
  projectName: 'ContextGenOS',

  onBrokenLinks: 'throw',
  onBrokenMarkdownLinks: 'warn',

  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  presets: [
    [
      'classic',
      /** @type {import('@docusaurus/preset-classic').Options} */
      ({
        docs: {
          sidebarPath: './sidebars.js',
          editUrl: 'https://github.com/Kisbjornssund/ContextGenOS/edit/main/docs/',
          showLastUpdateTime: true,
          showLastUpdateAuthor: true,
        },
        blog: false,
        theme: {
          customCss: './src/css/custom.css',
        },
      }),
    ],
  ],

  themeConfig:
    /** @type {import('@docusaurus/preset-classic').ThemeConfig} */
    ({
      navbar: {
        title: 'ContextGenOS',
        items: [
          {
            type: 'docSidebar',
            sidebarId: 'docs',
            position: 'left',
            label: 'Docs',
          },
          {
            href: 'https://github.com/Kisbjornssund/ContextGenOS',
            label: 'GitHub',
            position: 'right',
          },
          {
            href: 'https://discord.gg/contextgenos',
            label: 'Discord',
            position: 'right',
          },
        ],
      },
      footer: {
        style: 'dark',
        links: [
          {
            title: 'Docs',
            items: [
              { label: 'Quickstart', to: '/docs/quickstart' },
              { label: 'Writing a Collector', to: '/docs/contributing/writing-a-collector' },
              { label: 'Privacy Architecture', to: '/docs/privacy/architecture' },
            ],
          },
          {
            title: 'Community',
            items: [
              { label: 'Discord', href: 'https://discord.gg/contextgenos' },
              { label: 'GitHub Discussions', href: 'https://github.com/Kisbjornssund/ContextGenOS/discussions' },
              { label: 'Twitter / X', href: 'https://x.com/contextgenosdev' },
            ],
          },
          {
            title: 'Project',
            items: [
              { label: 'GitHub', href: 'https://github.com/Kisbjornssund/ContextGenOS' },
              { label: 'Manifesto', href: 'https://github.com/Kisbjornssund/ContextGenOS/blob/main/MANIFESTO.md' },
              { label: 'License (MIT)', href: 'https://github.com/Kisbjornssund/ContextGenOS/blob/main/LICENSE' },
            ],
          },
        ],
        copyright: `Copyright ${new Date().getFullYear()} Kimmo Isbjörnssund. MIT License.`,
      },
      prism: {
        additionalLanguages: ['rust', 'bash', 'toml', 'yaml', 'json'],
      },
      algolia: {
        // Add Algolia DocSearch credentials when available
        appId: 'PLACEHOLDER',
        apiKey: 'PLACEHOLDER',
        indexName: 'contextgenos',
      },
    }),
};

export default config;
