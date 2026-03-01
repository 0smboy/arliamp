use std::env;
use std::error::Error;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const OFF_SHADER: &str = r#"void mainImage(out vec4 fragColor, in vec2 fragCoord) {
    vec2 uv = fragCoord / iResolution.xy;
    fragColor = texture(iChannel0, uv);
}
"#;

const STATIC_SHADER: &str = r#"void mainImage(out vec4 fragColor, in vec2 fragCoord) {
    vec2 uv = fragCoord / iResolution.xy;
    vec2 crtUV = uv * 2.0 - 1.0;
    vec2 offset = crtUV.yx / 5.0;
    crtUV = crtUV + crtUV * offset * offset;
    crtUV = crtUV * 0.5 + 0.5;
    if (crtUV.x < 0.0 || crtUV.x > 1.0 || crtUV.y < 0.0 || crtUV.y > 1.0) {
        fragColor = vec4(0.0, 0.0, 0.0, 1.0);
        return;
    }
    float amount = 0.002;
    vec3 color;
    color.r = texture(iChannel0, vec2(crtUV.x + amount, crtUV.y)).r;
    color.g = texture(iChannel0, crtUV).g;
    color.b = texture(iChannel0, vec2(crtUV.x - amount, crtUV.y)).b;
    color -= sin(crtUV.y * 800.0 * 3.1415) * 0.04;
    color += sin(crtUV.y * 10.0 + iTime * 3.0) * 0.02;
    color += color * 0.3;
    float vignette = crtUV.x * crtUV.y * (1.0 - crtUV.x) * (1.0 - crtUV.y);
    color *= clamp(pow(16.0 * vignette, 0.25), 0.0, 1.0);
    fragColor = vec4(color, 1.0);
}
"#;

const CRAZY_SHADER: &str = r#"void mainImage(out vec4 fragColor, in vec2 fragCoord) {
    vec2 uv = fragCoord / iResolution.xy;
    vec2 centered = uv - 0.5;
    float beat = (sin(iTime * 2.5) + 1.0) * 0.5;
    float dist = length(centered);
    float centerScale = 1.0 - 0.45 * beat;
    float edgeScale = 1.0;
    float zDepth = mix(centerScale, edgeScale, dist * 1.5);
    float angle = sin(iTime * 1.0) * 0.02;
    mat2 rot = mat2(cos(angle), -sin(angle), sin(angle), cos(angle));
    centered = rot * centered * zDepth;
    vec2 final_uv = centered + 0.5;
    float edgeFade = smoothstep(0.5, 0.3, abs(uv.x - 0.5)) * smoothstep(0.5, 0.3, abs(uv.y - 0.5));

    if (final_uv.x < 0.0 || final_uv.x > 1.0 || final_uv.y < 0.0 || final_uv.y > 1.0) {
        fragColor = vec4(0.0);
        return;
    }

    float amount = 0.006 * beat;
    vec3 color;
    color.r = texture(iChannel0, vec2(final_uv.x + amount, final_uv.y)).r;
    color.g = texture(iChannel0, final_uv).g;
    color.b = texture(iChannel0, vec2(final_uv.x - amount, final_uv.y)).b;
    color *= edgeFade;
    color *= 1.1 + 0.4 * beat;

    fragColor = vec4(color, 1.0);
}
"#;

fn main() -> Result<(), Box<dyn Error>> {
    let raw_inputs: Vec<String> = env::args().skip(1).collect();
    let mut resolved_inputs = Vec::with_capacity(raw_inputs.len());
    for input in &raw_inputs {
        if looks_like_url(input) {
            resolved_inputs.push(input.clone());
            continue;
        }

        let resolved = fs::canonicalize(input)
            .map_err(|_| format!("arliamp: path not found: {}", Path::new(input).display()))?;

        if resolved.is_file() {
            validate_local_file_type(&resolved)?;
        } else if !resolved.is_dir() {
            return Err(format!("arliamp: unsupported path type: {}", resolved.display()).into());
        }

        resolved_inputs.push(resolved.to_string_lossy().to_string());
    }

    let tmux_bin = find_in_path("tmux")?;
    let unimatrix_bin = find_in_path("unimatrix")?;
    let rliamp_bin = find_in_path("rliamp")?;
    let _open_bin = find_in_path("open")?;

    let ghostty_app = Path::new("/Applications/Ghostty.app");
    let ghostty_bin = ghostty_app.join("Contents/MacOS/Ghostty");
    if !ghostty_bin.is_file() {
        return Err(format!("arliamp: Ghostty not found at {}", ghostty_bin.display()).into());
    }

    let runtime_dir = create_runtime_dir()?;
    let off_shader = runtime_dir.join("cyber-off.glsl");
    let static_shader = runtime_dir.join("cyber-static.glsl");
    let crazy_shader = runtime_dir.join("cyber-crazy.glsl");
    let state_file = runtime_dir.join("state");
    let toggle_script = runtime_dir.join("veo-toggle.sh");
    let tmux_conf = runtime_dir.join("tmux.conf");
    let center_script = runtime_dir.join("center.zsh");
    let runner_script = runtime_dir.join("runner.zsh");
    let ghostty_conf = runtime_dir.join("ghostty.conf");
    let tmux_socket = runtime_dir.join("tmux.sock");
    let session = format!("arliamp-{}", std::process::id());

    fs::write(&off_shader, OFF_SHADER)?;
    fs::write(&static_shader, STATIC_SHADER)?;
    fs::write(&crazy_shader, CRAZY_SHADER)?;
    fs::write(&state_file, "off\n")?;

    let toggle_body = format!(
        r#"#!/bin/sh
set -eu
state_file={state_file}
conf_file={conf_file}
ghostty_pid_file={ghostty_pid_file}
off_shader={off_shader}
static_shader={static_shader}
crazy_shader={crazy_shader}

current="off"
if [ -f "$state_file" ]; then
  current="$(cat "$state_file")"
fi

case "$current" in
  off)
    next="static"
    shader="$static_shader"
    label="STATIC GLOW"
    ;;
  static)
    next="crazy"
    shader="$crazy_shader"
    label="3D CRAZY"
    ;;
  crazy|*)
    next="off"
    shader="$off_shader"
    label="SHADER OFF"
    ;;
esac

tmp_conf="$conf_file.tmp"
awk -v shader="$shader" '
BEGIN {{updated=0}}
$1=="custom-shader" {{
  print "custom-shader = \"" shader "\""
  updated=1
  next
}}
{{print}}
END {{
  if (!updated) {{
    print "custom-shader = \"" shader "\""
  }}
}}
' "$conf_file" > "$tmp_conf"
mv "$tmp_conf" "$conf_file"

if [ -f "$ghostty_pid_file" ]; then
  ghostty_pid="$(cat "$ghostty_pid_file" 2>/dev/null || true)"
  if [ -n "$ghostty_pid" ] && kill -0 "$ghostty_pid" 2>/dev/null; then
    kill -USR2 "$ghostty_pid" 2>/dev/null || true
  fi
fi

printf '%s\n' "$next" > "$state_file"
tmux display-message "VEO: $label (v)"
"#,
        state_file = sh_quote(&state_file.to_string_lossy()),
        conf_file = sh_quote(&ghostty_conf.to_string_lossy()),
        ghostty_pid_file = sh_quote(&runtime_dir.join("ghostty.pid").to_string_lossy()),
        off_shader = sh_quote(&off_shader.to_string_lossy()),
        static_shader = sh_quote(&static_shader.to_string_lossy()),
        crazy_shader = sh_quote(&crazy_shader.to_string_lossy()),
    );
    write_executable(&toggle_script, &toggle_body)?;

    let tmux_body = format!(
        "set -g status off\n\
         set -g mouse on\n\
         set -g focus-events on\n\
         set -g detach-on-destroy on\n\
         set -g exit-empty on\n\
         set -g message-style \"bg=black,fg=green\"\n\
         set -g pane-border-style \"fg=black,bg=black\"\n\
         set -g pane-active-border-style \"fg=black,bg=black\"\n\
         set -g pane-border-status off\n\
         set -sg escape-time 0\n\
         bind -n v run-shell {toggle}\n\
         bind -n V run-shell {toggle}\n\
         bind v run-shell {toggle}\n",
        toggle = sh_quote(&toggle_script.to_string_lossy()),
    );
    fs::write(&tmux_conf, tmux_body)?;

    let center_body = format!(
        r#"#!/bin/sh
set -eu
export TERM=xterm-256color
export COLORTERM=truecolor
{rliamp} {inputs}
{tmux} -S {socket} kill-session -t {session} >/dev/null 2>&1 || true
"#,
        tmux = sh_quote(&tmux_bin.to_string_lossy()),
        socket = sh_quote(&tmux_socket.to_string_lossy()),
        session = sh_quote(&session),
        rliamp = sh_quote(&rliamp_bin.to_string_lossy()),
        inputs = resolved_inputs
            .iter()
            .map(|s| sh_quote(s))
            .collect::<Vec<_>>()
            .join(" "),
    );
    write_executable(&center_script, &center_body)?;

    let runner_body = format!(
        r##"#!/bin/sh
set -eu
TMUX_BIN={tmux}
UNIMATRIX_BIN={unimatrix}
CENTER_SCRIPT={center_script}
TMUX_SOCKET={socket}
TMUX_CONF={tmux_conf}
SESSION={session}
RUNTIME_DIR={runtime}
SIDE_CMD="\"$UNIMATRIX_BIN\" -s 96 -l o -c green"

cleanup() {{
  "$TMUX_BIN" -S "$TMUX_SOCKET" kill-session -t "$SESSION" >/dev/null 2>&1 || true
  "$TMUX_BIN" -S "$TMUX_SOCKET" kill-server >/dev/null 2>&1 || true
  rm -rf "$RUNTIME_DIR"
}}
trap cleanup EXIT INT TERM

detect_ghostty_pid() {{
  cur="$$"
  i=0
  while [ "$i" -lt 24 ]; do
    ppid="$(ps -o ppid= -p "$cur" 2>/dev/null | tr -d ' ')"
    [ -z "$ppid" ] && break
    [ "$ppid" -le 1 ] 2>/dev/null && break
    comm="$(ps -o comm= -p "$ppid" 2>/dev/null || true)"
    if printf '%s\n' "$comm" | grep -qi 'ghostty'; then
      printf '%s\n' "$ppid"
      return 0
    fi
    cur="$ppid"
    i=$((i + 1))
  done
  return 1
}}

if ghostty_pid="$(detect_ghostty_pid)"; then
  printf '%s\n' "$ghostty_pid" > "$RUNTIME_DIR/ghostty.pid"
fi

# Build a stable 3x3 stage before attach to avoid popup startup flicker.
"$TMUX_BIN" -S "$TMUX_SOCKET" -f "$TMUX_CONF" new-session -d -s "$SESSION" "$SIDE_CMD"
CENTER_PANE=$("$TMUX_BIN" -S "$TMUX_SOCKET" display-message -p -t "$SESSION":0.0 "#{{pane_id}}")
"$TMUX_BIN" -S "$TMUX_SOCKET" split-window -v -b -p 16 -t "$CENTER_PANE" "$SIDE_CMD"
"$TMUX_BIN" -S "$TMUX_SOCKET" split-window -v -p 19 -t "$CENTER_PANE" "$SIDE_CMD"
"$TMUX_BIN" -S "$TMUX_SOCKET" split-window -h -b -p 14 -t "$CENTER_PANE" "$SIDE_CMD"
"$TMUX_BIN" -S "$TMUX_SOCKET" split-window -h -p 16 -t "$CENTER_PANE" "$SIDE_CMD"
"$TMUX_BIN" -S "$TMUX_SOCKET" respawn-pane -k -t "$CENTER_PANE" "\"$CENTER_SCRIPT\""
"$TMUX_BIN" -S "$TMUX_SOCKET" select-pane -t "$CENTER_PANE"
"$TMUX_BIN" -S "$TMUX_SOCKET" attach -t "$SESSION"
"##,
        tmux = sh_quote(&tmux_bin.to_string_lossy()),
        unimatrix = sh_quote(&unimatrix_bin.to_string_lossy()),
        center_script = sh_quote(&center_script.to_string_lossy()),
        socket = sh_quote(&tmux_socket.to_string_lossy()),
        tmux_conf = sh_quote(&tmux_conf.to_string_lossy()),
        session = sh_quote(&session),
        runtime = sh_quote(&runtime_dir.to_string_lossy()),
    );
    write_executable(&runner_script, &runner_body)?;

    let ghostty_body = format!(
        "config-default-files = false\n\
         initial-command = direct:{}\n\
         custom-shader = \"{}\"\n\
         custom-shader-animation = true\n\
         confirm-close-surface = false\n\
         quit-after-last-window-closed = true\n\
         window-save-state = never\n\
         window-show-tab-bar = never\n\
         window-inherit-working-directory = false\n\
         window-inherit-font-size = false\n\
         shell-integration = none\n\
         background = #000000\n\
         fullscreen = true\n\
         macos-shortcuts = deny\n",
        runner_script.display(),
        off_shader.display(),
    );
    fs::write(&ghostty_conf, ghostty_body)?;

    let launch_status = Command::new("open")
        .arg("-na")
        .arg(ghostty_app)
        .arg("--args")
        .arg("--config-default-files=false")
        .arg("--fullscreen=true")
        .arg("--macos-non-native-fullscreen=true")
        .arg("--maximize=true")
        .arg(format!("--config-file={}", ghostty_conf.display()))
        .status()?;

    if !launch_status.success() {
        let _ = fs::remove_dir_all(&runtime_dir);
        return Err("arliamp: failed to launch Ghostty".into());
    }

    if resolved_inputs.is_empty() {
        println!("arliamp launched: provider mode (no input args)");
    } else {
        println!("arliamp launched: {}", resolved_inputs.join(" "));
    }
    Ok(())
}

fn looks_like_url(input: &str) -> bool {
    input.contains("://")
}

fn validate_local_file_type(path: &Path) -> Result<(), Box<dyn Error>> {
    const SUPPORTED_FILE_EXTENSIONS: &[&str] = &[
        "mp3", "wav", "flac", "ogg", "m4a", "aac", "m4b", "m4p", "alac", "wma", "opus", "m3u",
        "m3u8",
    ];

    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase())
        .ok_or_else(|| format!("arliamp: unsupported file type: {}", path.display()))?;

    if SUPPORTED_FILE_EXTENSIONS
        .iter()
        .any(|supported| *supported == ext)
    {
        return Ok(());
    }

    Err(format!(
        "arliamp: unsupported file type: {} (supported: {})",
        path.display(),
        SUPPORTED_FILE_EXTENSIONS.join(", ")
    )
    .into())
}

fn find_in_path(binary: &str) -> Result<PathBuf, Box<dyn Error>> {
    let path = env::var_os("PATH").ok_or_else(|| "PATH is not set".to_string())?;
    for dir in env::split_paths(&path) {
        let candidate = dir.join(binary);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err(format!("arliamp: missing dependency: {}", binary).into())
}

fn create_runtime_dir() -> Result<PathBuf, Box<dyn Error>> {
    let base = env::temp_dir();
    let pid = std::process::id();
    let epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let path = base.join(format!("arliamp.{}.{}", pid, epoch));
    fs::create_dir_all(&path)?;
    Ok(path)
}

fn write_executable(path: &Path, contents: &str) -> Result<(), Box<dyn Error>> {
    fs::write(path, contents)?;
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms)?;
    Ok(())
}

fn sh_quote(input: &str) -> String {
    let escaped = input.replace('\'', "'\"'\"'");
    format!("'{}'", escaped)
}
