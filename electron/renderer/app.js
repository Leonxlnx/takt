const profile = document.querySelector('#profile');
const volume = document.querySelector('#volume');
const volumeValue = document.querySelector('#volumeValue');
const autostart = document.querySelector('#autostart');
const status = document.querySelector('#status');
const apply = document.querySelector('#apply');
const restart = document.querySelector('#restart');
const stop = document.querySelector('#stop');
const openConfig = document.querySelector('#openConfig');

function readForm() {
  return {
    profile: profile.value,
    volume: Number(volume.value),
    autostart: autostart.checked
  };
}

function writeForm(config) {
  profile.value = config.profile || 'holy-panda';
  volume.value = config.volume ?? 65;
  volumeValue.textContent = `${volume.value}%`;
  autostart.checked = Boolean(config.autostart);
}

async function refreshStatus() {
  const running = await window.takt.status();
  status.textContent = running ? 'Running' : 'Stopped';
  status.style.background = running ? 'rgba(47, 125, 87, 0.24)' : 'rgba(216, 137, 88, 0.22)';
}

volume.addEventListener('input', () => {
  volumeValue.textContent = `${volume.value}%`;
});

apply.addEventListener('click', async () => {
  await window.takt.writeConfig(readForm());
  await refreshStatus();
});

restart.addEventListener('click', async () => {
  await window.takt.restart(readForm());
  await refreshStatus();
});

stop.addEventListener('click', async () => {
  await window.takt.stop();
  await refreshStatus();
});

openConfig.addEventListener('click', () => {
  window.takt.openConfig();
});

(async () => {
  writeForm(await window.takt.readConfig());
  await refreshStatus();
})();
