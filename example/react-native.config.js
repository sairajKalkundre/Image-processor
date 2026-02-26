const path = require('node:path');
const { withWorkspaceModule } = require('@craby/devkit');

const modulePackagePath = path.resolve(__dirname, '..');
const config = { assets: [
    "./assets",
  ]};

module.exports = withWorkspaceModule(config, modulePackagePath);
