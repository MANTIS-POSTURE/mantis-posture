import http from 'node:http';
import { spawn } from 'node:child_process';

const port = 5175;

function isViteServer() {
  return new Promise((resolve) => {
    const request = http.get({ host: 'localhost', port, path: '/@vite/client', timeout: 1500 }, (response) => {
      response.resume();
      resolve(response.statusCode === 200);
    });
    request.on('error', () => resolve(false));
    request.on('timeout', () => { request.destroy(); resolve(false); });
  });
}

if (await isViteServer()) {
  console.log(`Réutilisation du serveur Vite MANTIS sur le port ${port}.`);
  setInterval(() => {}, 60_000);
} else {
  const child = spawn(process.execPath, ['node_modules/vite/bin/vite.js', 'dev', '--port', String(port)], { stdio: 'inherit', shell: false });
  child.on('exit', (code) => process.exit(code ?? 1));
  for (const signal of ['SIGINT', 'SIGTERM']) process.on(signal, () => child.kill(signal));
}
