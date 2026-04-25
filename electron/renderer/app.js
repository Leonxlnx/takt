const profile = document.querySelector('#profile');
const volume = document.querySelector('#volume');
const volumeValue = document.querySelector('#volumeValue');
const autostart = document.querySelector('#autostart');
const status = document.querySelector('#status');
const apply = document.querySelector('#apply');
const restart = document.querySelector('#restart');
const stop = document.querySelector('#stop');
const openConfig = document.querySelector('#openConfig');
const profileHint = document.querySelector('#profileHint');

const hints = {
  'holy-panda': 'Rounded tactile thock with a clean bottom-out.',
  'oil-king': 'Deep, damped linear profile for a premium muted feel.',
  topre: 'Soft dome character with a low, rounded return.',
  'box-jade': 'Crisp clickbar-style snap with a bright top end.',
  'silent-tactile': 'Quiet tactile sound for lower-volume sessions.',
  'ink-black': 'Low thock with a smooth linear tail.',
  'nk-cream': 'Smooth pop with a slightly dry texture.',
  'buckling-spring': 'Vintage loud snap inspired by terminal boards.',
  'mx-black': 'Classic weighted linear clack.',
  'alps-blue': 'Bright click with sharper high frequencies.',
  ceramic: 'Clean, glassy clack with a fast transient.',
  terminal: 'Retro board tone with a longer body.',
  alpaca: 'Soft pop with a comfortable rounded attack.',
  typewriter: 'Sharp strike with a mechanical return character.'
};

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
  profileHint.textContent = hints[profile.value] || hints['holy-panda'];
}

async function refreshStatus() {
  const running = await window.takt.status();
  status.textContent = running ? 'Running' : 'Stopped';
  status.classList.toggle('is-stopped', !running);
}

volume.addEventListener('input', () => {
  volumeValue.textContent = `${volume.value}%`;
});

profile.addEventListener('change', () => {
  profileHint.textContent = hints[profile.value] || hints['holy-panda'];
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
