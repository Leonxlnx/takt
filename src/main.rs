#![windows_subsystem = "windows"]

use std::{
    env,
    f32::consts::TAU,
    io::{self, Write},
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Sender},
    },
    thread,
    time::Duration,
};

use anyhow::{Context, Result, anyhow};
use rodio::{OutputStream, Source};
use windows::Win32::{
    Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM},
    System::LibraryLoader::GetModuleHandleW,
    UI::{
        Input::KeyboardAndMouse::{VK_BACK, VK_ESCAPE, VK_RETURN, VK_SPACE, VK_TAB},
        WindowsAndMessaging::{
            CallNextHookEx, DispatchMessageW, GetMessageW, HHOOK, KBDLLHOOKSTRUCT, MSG,
            PostQuitMessage, SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx,
            WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
        },
    },
};

static KEY_EVENTS: OnceLock<Sender<KeyEvent>> = OnceLock::new();
static PRESSED_KEYS: OnceLock<Mutex<[bool; 256]>> = OnceLock::new();
static RUNNING: AtomicBool = AtomicBool::new(true);

#[derive(Clone, Copy, Debug)]
struct KeyEvent {
    vk_code: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SoundMode {
    Keys,
    Melody,
    Piano,
    Guitar,
    Chords,
}

impl SoundMode {
    fn named(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "keys" | "switch" | "switches" => Some(Self::Keys),
            "melody" | "song" | "tones" => Some(Self::Melody),
            "piano" => Some(Self::Piano),
            "guitar" | "pluck" => Some(Self::Guitar),
            "chords" | "chord" => Some(Self::Chords),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
struct Profile {
    name: &'static str,
    body_hz: f32,
    click_hz: f32,
    duration_ms: u64,
    body_decay: f32,
    click_decay: f32,
    noise: f32,
    volume: f32,
}

impl Profile {
    fn named(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "clean-muted" | "muted" => Some(Self {
                name: "clean-muted",
                body_hz: 78.0,
                click_hz: 470.0,
                duration_ms: 50,
                body_decay: 66.0,
                click_decay: 185.0,
                noise: 0.012,
                volume: 0.48,
            }),
            "clean-thock" | "thock" => Some(Self {
                name: "clean-thock",
                body_hz: 92.0,
                click_hz: 560.0,
                duration_ms: 64,
                body_decay: 50.0,
                click_decay: 160.0,
                noise: 0.016,
                volume: 0.46,
            }),
            "soft-linear" | "linear" => Some(Self {
                name: "soft-linear",
                body_hz: 112.0,
                click_hz: 690.0,
                duration_ms: 46,
                body_decay: 78.0,
                click_decay: 170.0,
                noise: 0.010,
                volume: 0.42,
            }),
            "studio-pop" | "pop" => Some(Self {
                name: "studio-pop",
                body_hz: 136.0,
                click_hz: 820.0,
                duration_ms: 42,
                body_decay: 86.0,
                click_decay: 155.0,
                noise: 0.008,
                volume: 0.40,
            }),
            "red" | "gateron-red" => Some(Self {
                name: "gateron-red",
                body_hz: 165.0,
                click_hz: 1850.0,
                duration_ms: 42,
                body_decay: 62.0,
                click_decay: 150.0,
                noise: 0.05,
                volume: 0.28,
            }),
            "panda" | "holy-panda" => Some(Self {
                name: "holy-panda",
                body_hz: 122.0,
                click_hz: 1180.0,
                duration_ms: 64,
                body_decay: 44.0,
                click_decay: 92.0,
                noise: 0.08,
                volume: 0.34,
            }),
            "blue" | "alps-blue" => Some(Self {
                name: "alps-blue",
                body_hz: 210.0,
                click_hz: 2600.0,
                duration_ms: 52,
                body_decay: 70.0,
                click_decay: 105.0,
                noise: 0.14,
                volume: 0.30,
            }),
            "navy" | "box-navy" => Some(Self {
                name: "box-navy",
                body_hz: 145.0,
                click_hz: 2200.0,
                duration_ms: 72,
                body_decay: 38.0,
                click_decay: 82.0,
                noise: 0.16,
                volume: 0.32,
            }),
            "topre" => Some(Self {
                name: "topre",
                body_hz: 96.0,
                click_hz: 760.0,
                duration_ms: 58,
                body_decay: 48.0,
                click_decay: 120.0,
                noise: 0.035,
                volume: 0.31,
            }),
            "cream" | "nk-cream" => Some(Self {
                name: "nk-cream",
                body_hz: 132.0,
                click_hz: 940.0,
                duration_ms: 66,
                body_decay: 42.0,
                click_decay: 86.0,
                noise: 0.055,
                volume: 0.33,
            }),
            "buckling" | "buckling-spring" => Some(Self {
                name: "buckling-spring",
                body_hz: 188.0,
                click_hz: 3100.0,
                duration_ms: 88,
                body_decay: 34.0,
                click_decay: 68.0,
                noise: 0.22,
                volume: 0.30,
            }),
            "ink" | "ink-black" => Some(Self {
                name: "ink-black",
                body_hz: 88.0,
                click_hz: 680.0,
                duration_ms: 74,
                body_decay: 36.0,
                click_decay: 110.0,
                noise: 0.045,
                volume: 0.35,
            }),
            "tealios" | "turquoise-tealios" => Some(Self {
                name: "turquoise-tealios",
                body_hz: 156.0,
                click_hz: 1320.0,
                duration_ms: 50,
                body_decay: 58.0,
                click_decay: 124.0,
                noise: 0.04,
                volume: 0.29,
            }),
            "alpaca" => Some(Self {
                name: "alpaca",
                body_hz: 118.0,
                click_hz: 860.0,
                duration_ms: 60,
                body_decay: 46.0,
                click_decay: 98.0,
                noise: 0.05,
                volume: 0.32,
            }),
            "typewriter" => Some(Self {
                name: "typewriter",
                body_hz: 240.0,
                click_hz: 3600.0,
                duration_ms: 96,
                body_decay: 30.0,
                click_decay: 62.0,
                noise: 0.26,
                volume: 0.27,
            }),
            "oil-king" | "oil" => Some(Self {
                name: "oil-king",
                body_hz: 82.0,
                click_hz: 610.0,
                duration_ms: 82,
                body_decay: 32.0,
                click_decay: 122.0,
                noise: 0.035,
                volume: 0.38,
            }),
            "mx-black" | "black" => Some(Self {
                name: "mx-black",
                body_hz: 104.0,
                click_hz: 790.0,
                duration_ms: 62,
                body_decay: 50.0,
                click_decay: 132.0,
                noise: 0.045,
                volume: 0.34,
            }),
            "box-jade" | "jade" => Some(Self {
                name: "box-jade",
                body_hz: 176.0,
                click_hz: 2850.0,
                duration_ms: 76,
                body_decay: 38.0,
                click_decay: 66.0,
                noise: 0.20,
                volume: 0.31,
            }),
            "silent-tactile" | "silent" => Some(Self {
                name: "silent-tactile",
                body_hz: 72.0,
                click_hz: 520.0,
                duration_ms: 46,
                body_decay: 74.0,
                click_decay: 190.0,
                noise: 0.018,
                volume: 0.36,
            }),
            "ceramic" => Some(Self {
                name: "ceramic",
                body_hz: 196.0,
                click_hz: 1720.0,
                duration_ms: 68,
                body_decay: 48.0,
                click_decay: 84.0,
                noise: 0.07,
                volume: 0.33,
            }),
            "terminal" => Some(Self {
                name: "terminal",
                body_hz: 154.0,
                click_hz: 2450.0,
                duration_ms: 92,
                body_decay: 28.0,
                click_decay: 74.0,
                noise: 0.18,
                volume: 0.29,
            }),
            _ => None,
        }
    }
}

fn main() -> Result<()> {
    let settings = Settings::from_args()?;
    let _ = writeln!(
        io::stdout(),
        "Takt running. profile={}, volume={:.0}%. Press Ctrl+C to quit.",
        settings.profile.name,
        settings.master_volume * 100.0
    );
    let _ = writeln!(
        io::stdout(),
        "Privacy: only virtual-key codes are observed; typed text is never stored or transmitted."
    );

    let (tx, rx) = mpsc::channel::<KeyEvent>();
    KEY_EVENTS
        .set(tx)
        .map_err(|_| anyhow!("keyboard event channel was already initialized"))?;
    PRESSED_KEYS
        .set(Mutex::new([false; 256]))
        .map_err(|_| anyhow!("keyboard state was already initialized"))?;

    let audio_settings = settings;
    let audio_thread = thread::spawn(move || run_audio(rx, audio_settings));

    ctrlc::set_handler(|| {
        RUNNING.store(false, Ordering::SeqCst);
        unsafe { PostQuitMessage(0) };
    })
    .context("failed to install Ctrl+C handler")?;

    run_keyboard_hook()?;
    audio_thread
        .join()
        .map_err(|_| anyhow!("audio thread panicked"))??;

    Ok(())
}

#[derive(Clone, Copy)]
struct Settings {
    mode: SoundMode,
    profile: Profile,
    master_volume: f32,
}

impl Settings {
    fn from_args() -> Result<Self> {
        let mut mode = SoundMode::Keys;
        let mut profile = Profile::named("clean-muted").unwrap();
        let mut master_volume = 0.75;
        let mut args = env::args().skip(1);

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--profile" | "-p" => {
                    let value = args
                        .next()
                        .ok_or_else(|| anyhow!("--profile requires a value"))?;
                    profile = Profile::named(&value).ok_or_else(|| {
                        anyhow!("unknown profile '{value}'. Use --help to list profiles")
                    })?;
                }
                "--volume" | "-v" => {
                    let value = args
                        .next()
                        .ok_or_else(|| anyhow!("--volume requires a value from 0 to 100"))?;
                    let parsed = value.parse::<f32>().context("volume must be a number")?;
                    master_volume = volume_to_gain(parsed);
                }
                "--mode" | "-m" => {
                    let value = args
                        .next()
                        .ok_or_else(|| anyhow!("--mode requires a value"))?;
                    mode = SoundMode::named(&value).ok_or_else(|| {
                        anyhow!(
                            "unknown mode '{value}'. Try keys, piano, guitar, melody, or chords"
                        )
                    })?;
                }
                "--help" | "-h" => {
                    print_help();
                    std::process::exit(0);
                }
                other => return Err(anyhow!("unknown argument '{other}'. Use --help.")),
            }
        }

        Ok(Self {
            mode,
            profile,
            master_volume,
        })
    }
}

fn print_help() {
    println!(
        "Usage: takt [--mode keys|piano|guitar|melody|chords] [--profile clean-muted] [--volume 75]\n\nClean profiles: clean-muted, clean-thock, soft-linear, studio-pop, silent-tactile"
    );
}

fn volume_to_gain(percent: f32) -> f32 {
    let normalized = (percent / 100.0).clamp(0.0, 1.0);
    if normalized <= 0.0 {
        0.0
    } else {
        0.16 + normalized.powf(0.68) * 2.84
    }
}

fn run_audio(rx: mpsc::Receiver<KeyEvent>, settings: Settings) -> Result<()> {
    let (_stream, handle) =
        OutputStream::try_default().context("failed to open default audio output")?;
    let mut melody_step = 0usize;
    let mut music = MusicGenerator::new(0x5eed_cafe);

    while RUNNING.load(Ordering::SeqCst) {
        let Ok(event) = rx.recv_timeout(Duration::from_millis(100)) else {
            continue;
        };

        let pan = key_pan(event.vk_code);
        match settings.mode {
            SoundMode::Keys => {
                let pitch = pitch_variation(event.vk_code);
                let source =
                    SwitchSource::new(settings.profile, settings.master_volume, pan, pitch);
                handle.play_raw(source.convert_samples())?;
            }
            SoundMode::Melody => {
                let source = MelodySource::new(music.next_note(), settings.master_volume, pan);
                melody_step = melody_step.wrapping_add(1);
                handle.play_raw(source.convert_samples())?;
            }
            SoundMode::Piano => {
                let source = PianoSource::new(music.next_note(), settings.master_volume, pan);
                melody_step = melody_step.wrapping_add(1);
                handle.play_raw(source.convert_samples())?;
            }
            SoundMode::Guitar => {
                let source = GuitarSource::new(
                    music.next_note() * 0.5,
                    melody_step,
                    settings.master_volume,
                    pan,
                );
                melody_step = melody_step.wrapping_add(1);
                handle.play_raw(source.convert_samples())?;
            }
            SoundMode::Chords => {
                let source = ChordSource::new(music.next_chord(), settings.master_volume, pan);
                melody_step = melody_step.wrapping_add(1);
                handle.play_raw(source.convert_samples())?;
            }
        }
    }

    Ok(())
}

fn run_keyboard_hook() -> Result<()> {
    let module = unsafe { GetModuleHandleW(None) }.context("failed to get module handle")?;
    let hook = unsafe {
        SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(low_level_keyboard_proc),
            HINSTANCE(module.0),
            0,
        )
    }
    .context("failed to install low-level keyboard hook")?;

    let mut msg = MSG::default();
    while RUNNING.load(Ordering::SeqCst) {
        let result = unsafe { GetMessageW(&mut msg, None, 0, 0) };
        if result.0 <= 0 {
            break;
        }
        unsafe {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    unsafe {
        let _ = UnhookWindowsHookEx(hook);
    }
    Ok(())
}

unsafe extern "system" fn low_level_keyboard_proc(
    code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if code >= 0 {
        let event = unsafe { *(l_param.0 as *const KBDLLHOOKSTRUCT) };
        let message = w_param.0 as u32;
        let key_index = event.vkCode as usize;

        if key_index < 256 && (message == WM_KEYDOWN || message == WM_SYSKEYDOWN) {
            if let Some(keys) = PRESSED_KEYS.get() {
                let Ok(mut keys) = keys.lock() else {
                    return unsafe { CallNextHookEx(HHOOK::default(), code, w_param, l_param) };
                };
                if keys[key_index] {
                    return unsafe { CallNextHookEx(HHOOK::default(), code, w_param, l_param) };
                }
                keys[key_index] = true;
            }

            if let Some(sender) = KEY_EVENTS.get() {
                let _ = sender.send(KeyEvent {
                    vk_code: event.vkCode,
                });
            }
        } else if key_index < 256 && (message == WM_KEYUP || message == WM_SYSKEYUP) {
            if let Some(keys) = PRESSED_KEYS.get() {
                if let Ok(mut keys) = keys.lock() {
                    keys[key_index] = false;
                }
            }
        }
    }

    unsafe { CallNextHookEx(HHOOK::default(), code, w_param, l_param) }
}

fn key_pan(vk_code: u32) -> f32 {
    match vk_code {
        0x51 | 0x41 | 0x5A | 0x31 | 0x32 => -0.85,
        0x57 | 0x53 | 0x58 | 0x33 => -0.55,
        0x45 | 0x44 | 0x43 | 0x34 => -0.32,
        0x52 | 0x46 | 0x56 | 0x54 | 0x47 | 0x42 | 0x35 | 0x36 => -0.12,
        0x59 | 0x48 | 0x4E | 0x55 | 0x4A | 0x4D | 0x37 => 0.12,
        0x49 | 0x4B | 0x38 => 0.32,
        0x4F | 0x4C | 0x39 => 0.55,
        0x50 | 0x30 => 0.85,
        code if code == VK_SPACE.0 as u32 => 0.0,
        code if code == VK_RETURN.0 as u32 => 0.68,
        code if code == VK_BACK.0 as u32 => 0.78,
        code if code == VK_TAB.0 as u32 || code == VK_ESCAPE.0 as u32 => -0.78,
        _ => 0.0,
    }
}

fn pitch_variation(vk_code: u32) -> f32 {
    let bucket = ((vk_code.wrapping_mul(2_654_435_761) >> 28) & 0x0f) as f32;
    0.94 + (bucket / 15.0) * 0.12
}

struct SwitchSource {
    profile: Profile,
    sample_rate: u32,
    total_frames: u32,
    frame: u32,
    channel: u16,
    pan: f32,
    pitch: f32,
    seed: u32,
    master_volume: f32,
}

impl SwitchSource {
    fn new(profile: Profile, master_volume: f32, pan: f32, pitch: f32) -> Self {
        let sample_rate = 48_000;
        let total_frames = (sample_rate as u64 * profile.duration_ms / 1_000) as u32;
        Self {
            profile,
            sample_rate,
            total_frames,
            frame: 0,
            channel: 0,
            pan,
            pitch,
            seed: 0x9e37_79b9,
            master_volume,
        }
    }

    fn mono_sample(&mut self) -> f32 {
        let t = self.frame as f32 / self.sample_rate as f32;
        let body_env = (-self.profile.body_decay * t).exp();
        let click_env = (-self.profile.click_decay * t).exp();
        let body_hz = self.profile.body_hz * self.pitch;
        let click_hz = self.profile.click_hz * self.pitch;
        let body = ((TAU * body_hz * t).sin()
            + (TAU * body_hz * 2.03 * t).sin() * 0.38
            + (TAU * body_hz * 3.07 * t).sin() * 0.16)
            * body_env;
        let click = ((TAU * click_hz * t).sin()
            + (TAU * click_hz * 1.51 * t).sin() * 0.24
            + (TAU * click_hz * 2.18 * t).sin() * 0.10)
            * click_env;
        let bottom_out = (TAU * (body_hz * 0.52) * t).sin() * (-95.0 * (t - 0.012).max(0.0)).exp();
        let noise = self.noise() * self.profile.noise * click_env;
        let sample = (body * 0.62 + click * 0.25 + bottom_out * 0.18 + noise)
            * self.profile.volume
            * self.master_volume;
        soft_clip(sample)
    }

    fn noise(&mut self) -> f32 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 17;
        self.seed ^= self.seed << 5;
        (self.seed as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

fn soft_clip(sample: f32) -> f32 {
    (sample * 1.45).tanh() * 0.96
}

const MAJOR_SCALE: [i32; 7] = [0, 2, 4, 5, 7, 9, 11];
const MINOR_SCALE: [i32; 7] = [0, 2, 3, 5, 7, 8, 10];
const PENTATONIC_SCALE: [i32; 5] = [0, 2, 4, 7, 9];
const ROOTS: [i32; 8] = [48, 50, 51, 53, 55, 56, 58, 60];
const PROGRESSIONS: [[usize; 4]; 8] = [
    [0, 4, 5, 3],
    [5, 3, 0, 4],
    [0, 5, 3, 4],
    [3, 4, 0, 5],
    [0, 2, 5, 4],
    [5, 4, 3, 4],
    [0, 4, 2, 5],
    [3, 0, 4, 5],
];

struct MusicGenerator {
    rng: u32,
    root_midi: i32,
    scale_id: usize,
    progression_id: usize,
    step: usize,
    last_degree: i32,
}

impl MusicGenerator {
    fn new(seed: u32) -> Self {
        Self {
            rng: seed,
            root_midi: 48,
            scale_id: 0,
            progression_id: 0,
            step: 0,
            last_degree: 0,
        }
    }

    fn next_note(&mut self) -> f32 {
        self.refresh_phrase();
        let chord_root = PROGRESSIONS[self.progression_id][(self.step / 4) % 4] as i32;
        let chord_tones = [chord_root, chord_root + 2, chord_root + 4, chord_root + 6];
        let degree = if self.chance(72) {
            chord_tones[self.range(chord_tones.len())]
        } else {
            let motion = [-2, -1, 1, 2, 3][self.range(5)];
            self.last_degree + motion
        };
        let octave = if self.chance(18) { 2 } else { 1 };
        self.last_degree = degree.clamp(0, 10);
        self.step = self.step.wrapping_add(1);
        self.degree_to_freq(degree, octave)
    }

    fn next_chord(&mut self) -> [f32; 5] {
        self.refresh_phrase();
        let root = PROGRESSIONS[self.progression_id][(self.step / 2) % 4] as i32;
        let extension = if self.chance(45) { 8 } else { 6 };
        self.step = self.step.wrapping_add(1);
        [
            self.degree_to_freq(root, -1),
            self.degree_to_freq(root, 0),
            self.degree_to_freq(root + 2, 0),
            self.degree_to_freq(root + 4, 0),
            self.degree_to_freq(root + extension, 0),
        ]
    }

    fn refresh_phrase(&mut self) {
        if self.step % 32 != 0 {
            return;
        }
        self.root_midi = ROOTS[self.range(ROOTS.len())];
        self.scale_id = self.range(3);
        self.progression_id = self.range(PROGRESSIONS.len());
        self.last_degree = PROGRESSIONS[self.progression_id][0] as i32;
    }

    fn degree_to_freq(&self, degree: i32, octave: i32) -> f32 {
        let scale = self.scale();
        let len = scale.len() as i32;
        let wrapped = degree.rem_euclid(len);
        let octave_shift = degree.div_euclid(len);
        midi_to_freq(self.root_midi + scale[wrapped as usize] + (octave + octave_shift) * 12)
    }

    fn scale(&self) -> &'static [i32] {
        match self.scale_id {
            1 => &MINOR_SCALE,
            2 => &PENTATONIC_SCALE,
            _ => &MAJOR_SCALE,
        }
    }

    fn chance(&mut self, percent: u32) -> bool {
        self.next_u32() % 100 < percent
    }

    fn range(&mut self, end: usize) -> usize {
        (self.next_u32() as usize) % end
    }

    fn next_u32(&mut self) -> u32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        self.rng
    }
}

fn midi_to_freq(midi: i32) -> f32 {
    440.0 * 2.0f32.powf((midi as f32 - 69.0) / 12.0)
}

struct MelodySource {
    sample_rate: u32,
    total_frames: u32,
    frame: u32,
    channel: u16,
    frequency: f32,
    pan: f32,
    master_volume: f32,
}

impl MelodySource {
    fn new(frequency: f32, master_volume: f32, pan: f32) -> Self {
        let sample_rate = 48_000;
        let total_frames = (sample_rate as f32 * 0.18) as u32;
        Self {
            sample_rate,
            total_frames,
            frame: 0,
            channel: 0,
            frequency,
            pan,
            master_volume,
        }
    }

    fn mono_sample(&self) -> f32 {
        let t = self.frame as f32 / self.sample_rate as f32;
        let attack = (t / 0.012).clamp(0.0, 1.0);
        let decay = (-8.5 * t).exp();
        let env = attack * decay;
        let fundamental = (TAU * self.frequency * t).sin();
        let octave = (TAU * self.frequency * 2.0 * t).sin() * 0.20;
        let fifth = (TAU * self.frequency * 1.5 * t).sin() * 0.12;
        soft_clip((fundamental + octave + fifth) * env * self.master_volume * 0.34)
    }
}

struct PianoSource {
    sample_rate: u32,
    total_frames: u32,
    frame: u32,
    channel: u16,
    frequency: f32,
    pan: f32,
    master_volume: f32,
}

impl PianoSource {
    fn new(frequency: f32, master_volume: f32, pan: f32) -> Self {
        let sample_rate = 48_000;
        let total_frames = (sample_rate as f32 * 0.58) as u32;
        Self {
            sample_rate,
            total_frames,
            frame: 0,
            channel: 0,
            frequency,
            pan,
            master_volume,
        }
    }

    fn mono_sample(&self) -> f32 {
        piano_note(
            self.frequency,
            self.frame,
            self.sample_rate,
            self.master_volume * 0.26,
        )
    }
}

impl Iterator for PianoSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let mono = self.mono_sample();
        stereo_next(
            &mut self.frame,
            &mut self.channel,
            self.total_frames,
            self.pan,
            mono,
        )
    }
}

impl Source for PianoSource {
    fn current_frame_len(&self) -> Option<usize> {
        Some(((self.total_frames - self.frame) * 2 - self.channel as u32) as usize)
    }

    fn channels(&self) -> u16 {
        2
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_millis(580))
    }
}

struct ChordSource {
    sample_rate: u32,
    total_frames: u32,
    frame: u32,
    channel: u16,
    chord: [f32; 5],
    pan: f32,
    master_volume: f32,
}

impl ChordSource {
    fn new(chord: [f32; 5], master_volume: f32, pan: f32) -> Self {
        let sample_rate = 48_000;
        let total_frames = (sample_rate as f32 * 0.78) as u32;
        Self {
            sample_rate,
            total_frames,
            frame: 0,
            channel: 0,
            chord,
            pan,
            master_volume,
        }
    }

    fn mono_sample(&self) -> f32 {
        let chord = self
            .chord
            .iter()
            .enumerate()
            .map(|(index, freq)| {
                let gain = if index == 0 { 0.14 } else { 0.085 };
                piano_note(
                    *freq,
                    self.frame,
                    self.sample_rate,
                    self.master_volume * gain,
                )
            })
            .sum::<f32>();
        soft_clip(chord)
    }
}

impl Iterator for ChordSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let mono = self.mono_sample();
        stereo_next(
            &mut self.frame,
            &mut self.channel,
            self.total_frames,
            self.pan,
            mono,
        )
    }
}

impl Source for ChordSource {
    fn current_frame_len(&self) -> Option<usize> {
        Some(((self.total_frames - self.frame) * 2 - self.channel as u32) as usize)
    }

    fn channels(&self) -> u16 {
        2
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_millis(780))
    }
}

struct GuitarSource {
    sample_rate: u32,
    total_frames: u32,
    frame: u32,
    channel: u16,
    buffer: Vec<f32>,
    index: usize,
    pan: f32,
    master_volume: f32,
}

impl GuitarSource {
    fn new(frequency: f32, step: usize, master_volume: f32, pan: f32) -> Self {
        let sample_rate = 48_000;
        let delay = (sample_rate as f32 / frequency).max(2.0) as usize;
        let mut seed = 0x1234_5678u32 ^ step as u32;
        let mut buffer = Vec::with_capacity(delay);
        for _ in 0..delay {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            buffer.push((seed as f32 / u32::MAX as f32) * 2.0 - 1.0);
        }

        Self {
            sample_rate,
            total_frames: (sample_rate as f32 * 0.62) as u32,
            frame: 0,
            channel: 0,
            buffer,
            index: 0,
            pan,
            master_volume,
        }
    }

    fn mono_sample(&mut self) -> f32 {
        let next_index = (self.index + 1) % self.buffer.len();
        let current = self.buffer[self.index];
        let next = self.buffer[next_index];
        let filtered = (current + next) * 0.497;
        self.buffer[self.index] = filtered;
        self.index = next_index;
        let t = self.frame as f32 / self.sample_rate as f32;
        soft_clip(current * (-2.2 * t).exp() * self.master_volume * 0.34)
    }
}

impl Iterator for GuitarSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.frame >= self.total_frames {
            return None;
        }

        let mono = self.mono_sample();
        let left_gain = ((1.0 - self.pan) * 0.5).sqrt();
        let right_gain = ((1.0 + self.pan) * 0.5).sqrt();
        let sample = if self.channel == 0 {
            mono * left_gain
        } else {
            mono * right_gain
        };

        self.channel += 1;
        if self.channel == 2 {
            self.channel = 0;
            self.frame += 1;
        }

        Some(sample)
    }
}

impl Source for GuitarSource {
    fn current_frame_len(&self) -> Option<usize> {
        Some(((self.total_frames - self.frame) * 2 - self.channel as u32) as usize)
    }

    fn channels(&self) -> u16 {
        2
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_millis(620))
    }
}

fn piano_note(frequency: f32, frame: u32, sample_rate: u32, gain: f32) -> f32 {
    let t = frame as f32 / sample_rate as f32;
    let attack = (t / 0.008).clamp(0.0, 1.0);
    let fast_decay = (-7.0 * t).exp();
    let slow_decay = (-1.7 * t).exp();
    let hammer = (TAU * frequency * 9.0 * t).sin() * (-70.0 * t).exp() * 0.035;
    let tone = (TAU * frequency * t).sin() * 1.0
        + (TAU * frequency * 2.0 * t).sin() * 0.44
        + (TAU * frequency * 3.01 * t).sin() * 0.22
        + (TAU * frequency * 4.02 * t).sin() * 0.11
        + (TAU * frequency * 5.0 * t).sin() * 0.055;
    soft_clip((tone * attack * (fast_decay * 0.72 + slow_decay * 0.28) + hammer) * gain)
}

fn stereo_next(
    frame: &mut u32,
    channel: &mut u16,
    total_frames: u32,
    pan: f32,
    mono: f32,
) -> Option<f32> {
    if *frame >= total_frames {
        return None;
    }

    let left_gain = ((1.0 - pan) * 0.5).sqrt();
    let right_gain = ((1.0 + pan) * 0.5).sqrt();
    let sample = if *channel == 0 {
        mono * left_gain
    } else {
        mono * right_gain
    };

    *channel += 1;
    if *channel == 2 {
        *channel = 0;
        *frame += 1;
    }

    Some(sample)
}

impl Iterator for MelodySource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.frame >= self.total_frames {
            return None;
        }

        let mono = self.mono_sample();
        let left_gain = ((1.0 - self.pan) * 0.5).sqrt();
        let right_gain = ((1.0 + self.pan) * 0.5).sqrt();
        let sample = if self.channel == 0 {
            mono * left_gain
        } else {
            mono * right_gain
        };

        self.channel += 1;
        if self.channel == 2 {
            self.channel = 0;
            self.frame += 1;
        }

        Some(sample)
    }
}

impl Source for MelodySource {
    fn current_frame_len(&self) -> Option<usize> {
        Some(((self.total_frames - self.frame) * 2 - self.channel as u32) as usize)
    }

    fn channels(&self) -> u16 {
        2
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_millis(180))
    }
}

impl Iterator for SwitchSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.frame >= self.total_frames {
            return None;
        }

        let mono = self.mono_sample();
        let left_gain = ((1.0 - self.pan) * 0.5).sqrt();
        let right_gain = ((1.0 + self.pan) * 0.5).sqrt();
        let sample = if self.channel == 0 {
            mono * left_gain
        } else {
            mono * right_gain
        };

        self.channel += 1;
        if self.channel == 2 {
            self.channel = 0;
            self.frame += 1;
        }

        Some(sample)
    }
}

impl Source for SwitchSource {
    fn current_frame_len(&self) -> Option<usize> {
        Some(((self.total_frames - self.frame) * 2 - self.channel as u32) as usize)
    }

    fn channels(&self) -> u16 {
        2
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_millis(self.profile.duration_ms))
    }
}
