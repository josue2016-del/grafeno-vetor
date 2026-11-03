//! Python macro system for Grafeno Vetor.
//!
//! Stores `.py` scripts in the settings folder (`macros/python/`), lists
//! them in the Python IDE docker, and runs them via a subprocess that
//! talks to the drawing through the VG object model (same as the VBA
//! macros).
//!
//! Execution model:
//!  - `python_exe()` finds a working Python (system first, then bundled).
//!  - `run_script(state, source)` writes the source to a temp file, spawns
//!    Python with `--`, reads stdout/stderr, translates errors into
//!    app messages.
//!  - The script sees a `grafeno` module exposing the same drawing
//!    primitives as the VG JS model: `rect()`, `ellipse()`, `text()`,
//!    `group()`, `active_layer()`, etc.

use crate::i18n::tr;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc::{channel, Receiver};

// ---------------------------------------------------------------------------
// Script storage
// ---------------------------------------------------------------------------

/// Where Python macros live on disk.
pub fn default_dir() -> Option<PathBuf> {
    crate::settings::config_dir().map(|d| d.join("macros").join("python"))
}

/// A single Python script stored on disk.
#[derive(Debug, Clone)]
pub struct PythonScript {
    pub name: String,
    pub path: PathBuf,
    pub source: String,
    /// Last error, if any.
    pub last_error: Option<String>,
    /// How long the last run took (ms).
    pub last_duration_ms: u64,
}

impl PythonScript {
    pub fn new(name: String, path: PathBuf, source: String) -> Self {
        Self {
            name,
            path,
            source,
            last_error: None,
            last_duration_ms: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// Script manager
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct PythonState {
    /// Scripts loaded from disk.
    pub scripts: Vec<PythonScript>,
    /// Whether we have scanned the folder at least once.
    pub loaded: bool,
    /// Output lines accumulated during the last run.
    pub output: Vec<String>,
    /// Whether a run is currently in progress.
    pub running: bool,
    /// Receiver for async output from the Python subprocess.
    pub rx: Option<Receiver<String>>,
}

impl PythonState {
    /// Drain any pending output from the subprocess channel.
    pub fn drain_output(&mut self) {
        if let Some(rx) = &self.rx {
            while let Ok(line) = rx.try_recv() {
                self.output.push(line);
            }
        }
    }

    /// Scan the macros folder and populate `scripts`.
    pub fn ensure_loaded(&mut self) {
        if self.loaded {
            return;
        }
        self.loaded = true;
        let Some(dir) = default_dir() else {
            return;
        };
        let Ok(rd) = std::fs::read_dir(&dir) else {
            return;
        };
        for entry in rd.flatten() {
            let path = entry.path();
            if path.extension().is_none_or(|e| e.to_ascii_lowercase() != "py") {
                continue;
            }
            let name = path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            if let Ok(source) = std::fs::read_to_string(&path) {
                self.scripts.push(PythonScript::new(name, path, source));
            }
        }
    }

    /// Create a new blank script and save it.
    pub fn create_new(&mut self, name: &str) -> Result<usize, String> {
        let dir = default_dir().ok_or(tr("python.no_settings_dir"))?;
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let stem: String = name
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '_' || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        let stem = if stem.is_empty() {
            "Untitled".to_string()
        } else {
            stem
        };
        let path = dir.join(format!("{stem}.py"));
        let source = format!(
            "# Grafeno Vetor – {stem}\n\
             # Use grafeno.rect(), grafeno.ellipse(), grafeno.text(), etc.\n\
             \n\
             def run():\n\
             \"\"\"Entry point called by Grafeno Vetor.\"\"\"\n\
             pass\n"
        );
        std::fs::write(&path, &source).map_err(|e| e.to_string())?;
        self.scripts.push(PythonScript::new(stem, path, source));
        Ok(self.scripts.len() - 1)
    }

    /// Save `idx`'s source back to disk.
    pub fn save_script(&mut self, idx: usize, source: &str) -> Result<(), String> {
        let s = self.scripts.get_mut(idx).ok_or("no script")?;
        std::fs::write(&s.path, source).map_err(|e| e.to_string())?;
        s.source = source.to_string();
        Ok(())
    }

    /// Delete `idx` from disk and memory.
    pub fn delete_script(&mut self, idx: usize) {
        if idx < self.scripts.len() {
            let path = self.scripts[idx].path.clone();
            let _ = std::fs::remove_file(&path);
            self.scripts.remove(idx);
        }
    }
}

// ---------------------------------------------------------------------------
// Python runtime discovery
// ---------------------------------------------------------------------------

/// Find a working `python.exe`. Checks in order:
/// 1. Bundled runtime (`assets/python/portable/`)
/// 2. System PATH (`python`, `python3`)
/// 3. Common install locations on Windows
pub fn python_exe() -> Option<PathBuf> {
    // 1. Bundled portable runtime inside the app directory.
    if let Ok(exe) = std::env::current_exe() {
        if let Some(app_dir) = exe.parent() {
            let bundled = app_dir.join("python-portable").join("python.exe");
            if bundled.exists() {
                return Some(bundled);
            }
        }
    }
    // Also check the executable's own directory (when run from a extracted zip).
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let bundled2 = parent.join("python-portable").join("python.exe");
            if bundled2.exists() {
                return Some(bundled2);
            }
        }
    }
    // 2. System PATH.
    for name in ["python", "python3"] {
        if which::which(name).is_ok() {
            return Some(which::which(name).ok()?);
        }
    }
    // 3. Common Windows install locations.
    let candidates = [
        r"C:\Python312\python.exe",
        r"C:\Python311\python.exe",
        r"C:\Python310\python.exe",
        r"C:\Python39\python.exe",
        r"C:\Python38\python.exe",
        r"C:\Users\Public\python-portable\python.exe",
    ];
    for c in &candidates {
        let p = PathBuf::from(c);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

/// Version string of the discovered Python, or `None`.
pub fn python_version() -> Option<String> {
    let exe = python_exe()?;
    let out = Command::new(&exe)
        .arg("--version")
        .output()
        .ok()?;
    if out.status.success() {
        Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Script execution
// ---------------------------------------------------------------------------

/// Run a Python script against the current drawing.
///
/// The script receives a `grafeno` module on `sys.modules` with drawing
/// primitives. It must define a `run()` function; if it does not, the
/// whole module body is executed as a one-shot script.
pub fn run_script(state: &mut PythonState, source: &str, script_name: &str) {
    let exe = match python_exe() {
        Some(e) => e,
        None => {
            state.output.push(tr("python.no_python"));
            return;
        }
    };

    // Write source to a temp file so Python can import it cleanly.
    let tmp_dir = std::env::temp_dir().join("grafeno-python");
    let _ = std::fs::create_dir_all(&tmp_dir);
    let script_path = tmp_dir.join(format!("{script_name}.py"));
    if let Err(e) = std::fs::write(&script_path, source) {
        state.output.push(format!("Write error: {e}"));
        return;
    }

    // Also write the VG bridge helper.
    let bridge_source = vg_bridge_source();
    let bridge_path = tmp_dir.join("_grafeno_bridge.py");
    let _ = std::fs::write(&bridge_path, bridge_source);

    state.running = true;
    state.output.clear();
    state
        .output
        .push(format!(">>> Running {} ...", script_name));

    // Set up channel for async output.
    let (tx, rx) = channel();
    state.rx = Some(rx);
    let name_owned = script_name.to_string();
    let exe_clone = exe.clone();
    let tmp_dir_clone = tmp_dir.clone();

    std::thread::spawn(move || {
        let start = std::time::Instant::now();
        let mut cmd = Command::new(&exe_clone);
        cmd.arg("-I")
            .arg("-B")
            .arg("-c")
            .arg(format!(
                "import sys; sys.path.insert(0, r'{}'); \
                 import _grafeno_bridge; \
                 import '{}'; \
                 if hasattr({}, 'run'): \
                     {}.run() \
                 else: \
                     pass",
                tmp_dir_clone.display(),
                name_owned,
                name_owned,
                name_owned,
            ))
            .current_dir(&tmp_dir_clone)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        log::info!("spawning Python: {:?}", cmd);

        match cmd.output() {
            Ok(out) => {
                let elapsed = start.elapsed().as_millis() as u64;
                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);
                for line in stdout.lines() {
                    let _ = tx.send(line.to_string());
                }
                if !stderr.is_empty() {
                    for line in stderr.lines() {
                        let _ = tx.send(format!("err: {line}"));
                    }
                }
                if !out.status.success() {
                    let code = out.status.code().unwrap_or(-1);
                    let _ = tx.send(format!("exited with code {code}"));
                }
                let _ = tx.send(format!("Done ({elapsed} ms)"));
            }
            Err(e) => {
                let _ = tx.send(format!("spawn failed: {e}"));
            }
        }
    });
}

/// The VG bridge module injected into every Python script.
/// Provides the same drawing API as the JS VG model.
fn vg_bridge_source() -> String {
    include_str!("scripting/python_bridge.py").to_string()
}

// ---------------------------------------------------------------------------
// Built-in macros
// ---------------------------------------------------------------------------

/// Returns the source of the built-in calendar generator macro.
pub fn builtin_calendar() -> &'static str {
    include_str!("scripting/python/macros/calendar_generator.py")
}

/// Returns the source of the built-in batch shape macro.
pub fn builtin_batch_shape() -> &'static str {
    include_str!("scripting/python/macros/batch_shape.py")
}
