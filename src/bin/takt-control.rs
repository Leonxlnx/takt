#![windows_subsystem = "windows"]

use std::{
    cell::RefCell,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    rc::Rc,
};

use native_windows_gui as nwg;

const PROFILES: &[(&str, &str)] = &[
    ("holy-panda", "Holy Panda - tactile thock"),
    ("oil-king", "Oil King - deep linear"),
    ("topre", "Topre - soft dome"),
    ("box-jade", "Box Jade - crisp click"),
    ("silent-tactile", "Silent Tactile - muted"),
    ("ink-black", "Ink Black - low thock"),
    ("nk-cream", "NK Cream - smooth pop"),
    ("buckling-spring", "Buckling Spring - vintage"),
    ("mx-black", "MX Black - classic linear"),
    ("alps-blue", "Alps Blue - bright click"),
    ("ceramic", "Ceramic - clean clack"),
    ("terminal", "Terminal - retro board"),
    ("alpaca", "Alpaca - soft pop"),
    ("typewriter", "Typewriter - sharp strike"),
];

#[derive(Clone)]
struct Settings {
    profile: String,
    volume: u32,
    autostart: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            profile: "holy-panda".to_string(),
            volume: 65,
            autostart: true,
        }
    }
}

fn main() {
    if let Err(error) = run() {
        nwg::modal_error_message(&nwg::Window::default(), "Takt", &error);
    }
}

fn run() -> Result<(), String> {
    nwg::init().map_err(|error| error.to_string())?;
    nwg::Font::set_global_family("Segoe UI").map_err(|error| error.to_string())?;

    let app = Rc::new(RefCell::new(TaktApp::new(load_settings())?));
    app.borrow().refresh_status();

    let events_app = Rc::clone(&app);
    let window_handle = app.borrow().window.handle;
    let handler =
        nwg::full_bind_event_handler(&window_handle, move |event, _event_data, handle| {
            if event == nwg::Event::OnWindowClose {
                nwg::stop_thread_dispatch();
                return;
            }

            let app = events_app.borrow();
            if event == nwg::Event::OnButtonClick && handle == app.apply.handle {
                let settings = app.read_settings();
                let _ = save_settings(&settings);
                let _ = set_autostart(settings.autostart);
                app.set_status("Settings saved");
            }

            if event == nwg::Event::OnButtonClick && handle == app.restart.handle {
                let settings = app.read_settings();
                let _ = save_settings(&settings);
                let _ = set_autostart(settings.autostart);
                let _ = stop_engine();
                let _ = start_engine(&settings);
                app.set_status("Status: running");
            }

            if event == nwg::Event::OnButtonClick && handle == app.stop.handle {
                let _ = stop_engine();
                app.set_status("Status: stopped");
            }

            if event == nwg::Event::OnHorizontalScroll && handle == app.volume.handle {
                app.update_volume_label();
            }
        });

    nwg::dispatch_thread_events();
    nwg::unbind_event_handler(&handler);
    Ok(())
}

#[allow(dead_code)]
struct TaktApp {
    window: nwg::Window,
    title: nwg::Label,
    subtitle: nwg::Label,
    status: nwg::Label,
    profile_label: nwg::Label,
    profile: nwg::ComboBox<&'static str>,
    volume_label: nwg::Label,
    volume: nwg::TrackBar,
    autostart: nwg::CheckBox,
    apply: nwg::Button,
    restart: nwg::Button,
    stop: nwg::Button,
    footer: nwg::Label,
    _title_font: nwg::Font,
    _label_font: nwg::Font,
    _body_font: nwg::Font,
}

impl TaktApp {
    fn new(settings: Settings) -> Result<Self, String> {
        let title_font = font(30, 700)?;
        let label_font = font(15, 700)?;
        let body_font = font(15, 400)?;

        let mut window = nwg::Window::default();
        let mut title = nwg::Label::default();
        let mut subtitle = nwg::Label::default();
        let mut status = nwg::Label::default();
        let mut profile_label = nwg::Label::default();
        let mut profile = nwg::ComboBox::default();
        let mut volume_label = nwg::Label::default();
        let mut volume = nwg::TrackBar::default();
        let mut autostart = nwg::CheckBox::default();
        let mut apply = nwg::Button::default();
        let mut restart = nwg::Button::default();
        let mut stop = nwg::Button::default();
        let mut footer = nwg::Label::default();

        nwg::Window::builder()
            .size((520, 430))
            .position((300, 300))
            .title("Takt")
            .flags(nwg::WindowFlags::WINDOW)
            .build(&mut window)
            .map_err(|error| error.to_string())?;

        nwg::Label::builder()
            .text("Takt")
            .parent(&window)
            .position((42, 34))
            .size((420, 52))
            .font(Some(&title_font))
            .build(&mut title)
            .map_err(|error| error.to_string())?;

        nwg::Label::builder()
            .text("Clean keyboard sound for Windows")
            .parent(&window)
            .position((45, 90))
            .size((420, 24))
            .font(Some(&body_font))
            .build(&mut subtitle)
            .map_err(|error| error.to_string())?;

        nwg::Label::builder()
            .text("Status")
            .parent(&window)
            .position((45, 136))
            .size((420, 24))
            .font(Some(&label_font))
            .build(&mut status)
            .map_err(|error| error.to_string())?;

        nwg::Label::builder()
            .text("Sound")
            .parent(&window)
            .position((45, 180))
            .size((420, 24))
            .font(Some(&label_font))
            .build(&mut profile_label)
            .map_err(|error| error.to_string())?;

        let labels = PROFILES.iter().map(|(_, label)| *label).collect::<Vec<_>>();
        nwg::ComboBox::builder()
            .parent(&window)
            .position((45, 208))
            .size((420, 32))
            .collection(labels)
            .selected_index(Some(profile_index(&settings.profile)))
            .build(&mut profile)
            .map_err(|error| error.to_string())?;

        nwg::Label::builder()
            .text(&format!("Volume: {}%", settings.volume))
            .parent(&window)
            .position((45, 264))
            .size((420, 24))
            .font(Some(&label_font))
            .build(&mut volume_label)
            .map_err(|error| error.to_string())?;

        nwg::TrackBar::builder()
            .parent(&window)
            .position((44, 296))
            .size((420, 34))
            .range(Some(0..100))
            .pos(Some(settings.volume as usize))
            .build(&mut volume)
            .map_err(|error| error.to_string())?;

        nwg::CheckBox::builder()
            .text("Start with Windows")
            .parent(&window)
            .position((45, 342))
            .size((190, 28))
            .check_state(if settings.autostart {
                nwg::CheckBoxState::Checked
            } else {
                nwg::CheckBoxState::Unchecked
            })
            .build(&mut autostart)
            .map_err(|error| error.to_string())?;

        nwg::Button::builder()
            .text("Apply")
            .parent(&window)
            .position((250, 338))
            .size((76, 34))
            .build(&mut apply)
            .map_err(|error| error.to_string())?;

        nwg::Button::builder()
            .text("Restart")
            .parent(&window)
            .position((338, 338))
            .size((76, 34))
            .build(&mut restart)
            .map_err(|error| error.to_string())?;

        nwg::Button::builder()
            .text("Stop")
            .parent(&window)
            .position((426, 338))
            .size((52, 34))
            .build(&mut stop)
            .map_err(|error| error.to_string())?;

        nwg::Label::builder()
            .text("Local only. No telemetry. No typed text is stored.")
            .parent(&window)
            .position((45, 388))
            .size((420, 24))
            .build(&mut footer)
            .map_err(|error| error.to_string())?;

        Ok(Self {
            window,
            title,
            subtitle,
            status,
            profile_label,
            profile,
            volume_label,
            volume,
            autostart,
            apply,
            restart,
            stop,
            footer,
            _title_font: title_font,
            _label_font: label_font,
            _body_font: body_font,
        })
    }

    fn read_settings(&self) -> Settings {
        let profile_index = self.profile.selection().unwrap_or(0);
        let profile = PROFILES
            .get(profile_index)
            .map(|(id, _)| *id)
            .unwrap_or("holy-panda")
            .to_string();
        let volume = self.volume.pos().clamp(0, 100) as u32;
        let autostart = self.autostart.check_state() == nwg::CheckBoxState::Checked;
        self.update_volume_label();

        Settings {
            profile,
            volume,
            autostart,
        }
    }

    fn refresh_status(&self) {
        if is_running() {
            self.status.set_text("Status: running");
        } else {
            self.status.set_text("Status: stopped");
        }
    }

    fn set_status(&self, text: &str) {
        self.status.set_text(text);
    }

    fn update_volume_label(&self) {
        self.volume_label
            .set_text(&format!("Volume: {}%", self.volume.pos()));
    }
}

fn font(size: u32, weight: u32) -> Result<nwg::Font, String> {
    let mut font = nwg::Font::default();
    nwg::Font::builder()
        .family("Segoe UI")
        .size(size)
        .weight(weight)
        .build(&mut font)
        .map_err(|error| error.to_string())?;
    Ok(font)
}

fn profile_index(profile: &str) -> usize {
    PROFILES
        .iter()
        .position(|(id, _)| *id == profile)
        .unwrap_or(0)
}

fn load_settings() -> Settings {
    let path = config_path();
    let Ok(text) = fs::read_to_string(path) else {
        return Settings::default();
    };

    let mut settings = Settings::default();
    for line in text.lines() {
        let clean = line.trim().trim_end_matches(',');
        if clean.starts_with("\"profile\"") {
            if let Some(value) = json_string_value(clean) {
                settings.profile = value;
            }
        } else if clean.starts_with("\"volume\"") {
            if let Some(value) = clean
                .split(':')
                .nth(1)
                .and_then(|value| value.trim().parse().ok())
            {
                settings.volume = value;
            }
        } else if clean.starts_with("\"autostart\"") {
            settings.autostart = clean.contains("true");
        }
    }
    settings
}

fn save_settings(settings: &Settings) -> Result<(), String> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(
        path,
        format!(
            "{{\n  \"profile\": \"{}\",\n  \"volume\": {},\n  \"autostart\": {}\n}}\n",
            settings.profile, settings.volume, settings.autostart
        ),
    )
    .map_err(|error| error.to_string())
}

fn json_string_value(line: &str) -> Option<String> {
    line.split(':')
        .nth(1)
        .map(|value| value.trim().trim_matches('"').to_string())
}

fn config_path() -> PathBuf {
    appdata_dir().join("Takt").join("config.json")
}

fn install_dir() -> PathBuf {
    localappdata_dir().join("Takt")
}

fn startup_shortcut() -> PathBuf {
    appdata_dir()
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs")
        .join("Startup")
        .join("Takt.lnk")
}

fn appdata_dir() -> PathBuf {
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn localappdata_dir() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn engine_path() -> PathBuf {
    install_dir().join("takt.exe")
}

fn launcher_path() -> PathBuf {
    install_dir().join("scripts").join("launch-hidden.vbs")
}

fn start_engine(settings: &Settings) -> Result<(), String> {
    Command::new(engine_path())
        .args([
            "--profile",
            &settings.profile,
            "--volume",
            &settings.volume.to_string(),
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn stop_engine() -> Result<(), String> {
    let _ = Command::new("taskkill")
        .args(["/IM", "takt.exe", "/F"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    Ok(())
}

fn is_running() -> bool {
    Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq takt.exe"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).contains("takt.exe"))
        .unwrap_or(false)
}

fn set_autostart(enabled: bool) -> Result<(), String> {
    let shortcut = startup_shortcut();
    if !enabled {
        let _ = fs::remove_file(shortcut);
        return Ok(());
    }

    if let Some(parent) = shortcut.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    create_shortcut(&shortcut, &launcher_path(), "Start Takt with Windows")
}

fn create_shortcut(path: &Path, target: &Path, description: &str) -> Result<(), String> {
    let windir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string());
    let script = format!(
        "$s=New-Object -ComObject WScript.Shell;$l=$s.CreateShortcut('{}');$l.TargetPath='{}\\System32\\wscript.exe';$l.Arguments='\"{}\"';$l.Description='{}';$l.Save()",
        path.display(),
        windir,
        target.display(),
        description
    );

    Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &script,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|_| ())
        .map_err(|error| error.to_string())
}
