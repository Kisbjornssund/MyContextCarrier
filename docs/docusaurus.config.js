// @ts-check

/** @type {import('@docusaurus/types').Config} */
const config = {
  title: 'MyContextPort',
  tagline: 'Your personal AI memory layer. Local. Private. Yours.',
  favicon: 'img/favicon.ico',

  url: 'https://kisbjornssund.github.io',
  baseUrl: '/MyContextPort/',

  organizationName: 'Kisbjornssund',
  projectName: 'MyContextPort',

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
          editUrl: 'https://github.com/Kisbjornssund/MyContextPort/edit/main/docs/',
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
        title: 'MyContextPort',
        items: [
          {
            type: 'docSidebar',
            sidebarId: 'docs',
            position: 'left',
            label: 'Docs',
          },
          {
            href: 'https://github.com/Kisbjornssund/MyContextPort',
            label: 'GitHub',
            position: 'right',
          },
          {
            href: 'https://discord.gg/NvqtCBRr',
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
              { label: 'Contributing', to: '/docs/contributing/first-contribution' },
            ],
          },
          {
            title: 'Community',
            items: [
              { label: 'Discord', href: 'https://discord.gg/NvqtCBRr' },
              { label: 'GitHub Discussions', href: 'https://github.com/Kisbjornssund/MyContextPort/discussions' },
              { label: 'Twitter / X', href: 'https://x.com/mycontextportdev' },
            ],
          },
          {
            title: 'Project',
            items: [
              { label: 'GitHub', href: 'https://github.com/Kisbjornssund/MyContextPort' },
              { label: 'Manifesto', href: 'https://github.com/Kisbjornssund/MyContextPort/blob/main/MANIFESTO.md' },
              { label: 'License (MIT)', href: 'https://github.com/Kisbjornssund/MyContextPort/blob/main/LICENSE' },
            ],
          },
        ],
        copyright: `Copyright ${new Date().getFullYear()} Kimmo Isbjörnssund. MIT License.`,
      },
      prism: {
        additionalLanguages: ['rust', 'bash', 'toml', 'yaml', 'json'],
      },
    }),
};

export default config;
