const { app, BrowserWindow, ipcMain, shell } = require('electron');
const fs = require('node:fs');
const path = require('node:path');
const { spawn, execFile } = require('node:child_process');

const appName = 'Takt';
const rootDir = path.resolve(__dirname, '..');
const installDir = path.join(process.env.LOCALAPPDATA || rootDir, appName);
const configDir = path.join(process.env.APPDATA || rootDir, appName);
const configPath = path.join(configDir, 'config.json');
const enginePath = resolveEnginePath();

let mainWindow;

function resolveEnginePath() {
  const installed = path.join(installDir, 'takt.exe');
  const dev = path.join(rootDir, 'target', 'release', 'takt.exe');
  return fs.existsSync(installed) ? installed : dev;
}

function defaultConfig() {
  return {
    profile: 'holy-panda',
    volume: 65,
    autostart: true
  };
}

function readConfig() {
  try {
    return { ...defaultConfig(), ...JSON.parse(fs.readFileSync(configPath, 'utf8')) };
  } catch {
    return defaultConfig();
  }
}

function writeConfig(config) {
  fs.mkdirSync(configDir, { recursive: true });
  fs.writeFileSync(configPath, `${JSON.stringify(config, null, 2)}\n`);
}

function startEngine(config = readConfig()) {
  if (!fs.existsSync(enginePath)) {
    throw new Error(`Sound engine not found: ${enginePath}`);
  }

  const child = spawn(enginePath, ['--profile', config.profile, '--volume', String(config.volume)], {
    detached: true,
    stdio: 'ignore',
    windowsHide: true
  });
  child.unref();
}

function stopEngine() {
  return new Promise((resolve) => {
    execFile('taskkill', ['/IM', 'takt.exe', '/F'], { windowsHide: true }, () => resolve());
  });
}

function isRunning() {
  return new Promise((resolve) => {
    execFile('tasklist', ['/FI', 'IMAGENAME eq takt.exe'], { windowsHide: true }, (_error, stdout = '') => {
      resolve(stdout.toLowerCase().includes('takt.exe'));
    });
  });
}

function createWindow() {
  mainWindow = new BrowserWindow({
    width: 900,
    height: 640,
    minWidth: 780,
    minHeight: 560,
    title: 'Takt',
    backgroundColor: '#f4efe5',
    autoHideMenuBar: true,
    webPreferences: {
      preload: path.join(__dirname, 'preload.cjs'),
      contextIsolation: true,
      nodeIntegration: false
    }
  });

  mainWindow.loadFile(path.join(__dirname, 'renderer', 'index.html'));
}

app.whenReady().then(() => {
  createWindow();

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
  });
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') app.quit();
});

ipcMain.handle('config:read', async () => readConfig());
ipcMain.handle('config:write', async (_event, config) => {
  writeConfig(config);
  return readConfig();
});
ipcMain.handle('engine:start', async (_event, config) => {
  startEngine(config);
  return isRunning();
});
ipcMain.handle('engine:stop', async () => {
  await stopEngine();
  return isRunning();
});
ipcMain.handle('engine:restart', async (_event, config) => {
  writeConfig(config);
  await stopEngine();
  startEngine(config);
  return isRunning();
});
ipcMain.handle('engine:status', async () => isRunning());
ipcMain.handle('path:open-config', async () => {
  fs.mkdirSync(configDir, { recursive: true });
  if (!fs.existsSync(configPath)) writeConfig(defaultConfig());
  shell.showItemInFolder(configPath);
});
