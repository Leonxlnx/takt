use std::{
    env,
    f32::consts::TAU,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Sender},
        OnceLock,
    },
    thread,
    time::Duration,
};

use anyhow::{anyhow, Context, Result};
use rodio::{OutputStream, Source};
use windows::{
    Win32::{
        Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Input::KeyboardAndMouse::{VK_BACK, VK_ESCAPE, VK_RETURN, VK_SPACE, VK_TAB},
            WindowsAndMessaging::{
                CallNextHookEx, DispatchMessageW, GetMessageW, PostQuitMessage, SetWindowsHookExW,
                TranslateMessage, UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT, MSG,
                WH_KEYBOARD_LL, WM_KEYDOWN, WM_SYSKEYDOWN,
            },
        },
    },
};

static KEY_EVENTS: OnceLock<Sender<KeyEvent>> = OnceLock::new();
static RUNNING: AtomicBool = AtomicBool::new(true);

#[derive(Clone, Copy, Debug)]
struct KeyEvent {
    vk_code: u32,
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
            _ => None,
        }
    }
}

fn main() -> Result<()> {
    let settings = Settings::from_args()?;
    println!(
        "Keyme running. profile={}, volume={:.0}%. Press Ctrl+C to quit.",
        settings.profile.name,
        settings.master_volume * 100.0
    );
    println!("Privacy: only virtual-key codes are observed; typed text is never stored or transmitted.");

    let (tx, rx) = mpsc::channel::<KeyEvent>();
    KEY_EVENTS
        .set(tx)
        .map_err(|_| anyhow!("keyboard event channel was already initialized"))?;

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
    profile: Profile,
    master_volume: f32,
}

impl Settings {
    fn from_args() -> Result<Self> {
        let mut profile = Profile::named("holy-panda").unwrap();
        let mut master_volume = 0.75;
        let mut args = env::args().skip(1);

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--profile" | "-p" => {
                    let value = args
                        .next()
                        .ok_or_else(|| anyhow!("--profile requires a value"))?;
                    profile = Profile::named(&value).ok_or_else(|| {
                        anyhow!(
                            "unknown profile '{value}'. Try red, holy-panda, alps-blue, box-navy, or topre"
                        )
                    })?;
                }
                "--volume" | "-v" => {
                    let value = args
                        .next()
                        .ok_or_else(|| anyhow!("--volume requires a value from 0 to 100"))?;
                    let parsed = value.parse::<f32>().context("volume must be a number")?;
                    master_volume = (parsed / 100.0).clamp(0.0, 1.0);
                }
                "--help" | "-h" => {
                    print_help();
                    std::process::exit(0);
                }
                other => return Err(anyhow!("unknown argument '{other}'. Use --help.")),
            }
        }

        Ok(Self {
            profile,
            master_volume,
        })
    }
}

fn print_help() {
    println!(
        "Usage: keyme [--profile holy-panda] [--volume 75]\n\nProfiles: red, holy-panda, alps-blue, box-navy, topre"
    );
}

fn run_audio(rx: mpsc::Receiver<KeyEvent>, settings: Settings) -> Result<()> {
    let (_stream, handle) = OutputStream::try_default().context("failed to open default audio output")?;

    while RUNNING.load(Ordering::SeqCst) {
        let Ok(event) = rx.recv_timeout(Duration::from_millis(100)) else {
            continue;
        };

        let pan = key_pan(event.vk_code);
        let pitch = pitch_variation(event.vk_code);
        let source = SwitchSource::new(settings.profile, settings.master_volume, pan, pitch);
        handle.play_raw(source.convert_samples())?;
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
    if code >= 0 && (w_param.0 as u32 == WM_KEYDOWN || w_param.0 as u32 == WM_SYSKEYDOWN) {
        let event = unsafe { *(l_param.0 as *const KBDLLHOOKSTRUCT) };
        if let Some(sender) = KEY_EVENTS.get() {
            let _ = sender.send(KeyEvent {
                vk_code: event.vkCode,
            });
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
        let body = (TAU * self.profile.body_hz * self.pitch * t).sin() * body_env;
        let click = (TAU * self.profile.click_hz * self.pitch * t).sin() * click_env;
        let noise = self.noise() * self.profile.noise * click_env;
        (body * 0.72 + click * 0.28 + noise) * self.profile.volume * self.master_volume
    }

    fn noise(&mut self) -> f32 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 17;
        self.seed ^= self.seed << 5;
        (self.seed as f32 / u32::MAX as f32) * 2.0 - 1.0
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
