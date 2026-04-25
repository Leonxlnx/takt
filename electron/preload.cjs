const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('takt', {
  readConfig: () => ipcRenderer.invoke('config:read'),
  writeConfig: (config) => ipcRenderer.invoke('config:write', config),
  start: (config) => ipcRenderer.invoke('engine:start', config),
  stop: () => ipcRenderer.invoke('engine:stop'),
  restart: (config) => ipcRenderer.invoke('engine:restart', config),
  status: () => ipcRenderer.invoke('engine:status'),
  openConfig: () => ipcRenderer.invoke('path:open-config')
});
