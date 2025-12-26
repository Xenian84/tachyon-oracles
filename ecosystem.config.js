module.exports = {
  apps: [
    {
      name: 'tachyon-signer',
      script: 'dist/index.js',
      cwd: '/root/tachyon-oracles/signer',
      env: {
        NODE_ENV: 'production',
      },
      error_file: '/root/tachyon-oracles/logs/signer-error.log',
      out_file: '/root/tachyon-oracles/logs/signer-out.log',
      time: true,
    },
    {
      name: 'tachyon-relayer',
      script: 'dist/index.js',
      cwd: '/root/tachyon-oracles/relayer',
      env: {
        NODE_ENV: 'production',
        RELAYER_PORT: '7777',
      },
      error_file: '/root/tachyon-oracles/logs/relayer-error.log',
      out_file: '/root/tachyon-oracles/logs/relayer-out.log',
      time: true,
    },
  ],
};
