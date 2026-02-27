// @ts-check

/** @type {import('@docusaurus/plugin-content-docs').SidebarsConfig} */
const sidebars = {
  docs: [
    {
      type: 'doc',
      id: 'quickstart',
      label: 'Quickstart',
    },
    {
      type: 'category',
      label: 'Contributing',
      items: [
        'contributing/first-contribution',
        'contributing/writing-a-collector',
      ],
    },
  ],
};

module.exports = sidebars;
