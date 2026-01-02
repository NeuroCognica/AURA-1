import sys
import traceback

def excepthook(exc_type, exc, tb):
    print("FATAL LAUNCHER EXCEPTION", file=sys.stderr)
    traceback.print_exception(exc_type, exc, tb)

sys.excepthook = excepthook

import tkinter as tk
from tkinter import scrolledtext
import json
import subprocess
import threading
import os
import sys
import signal
import re
import tempfile
from urllib.parse import urlparse

# --- CONFIGURATION ---
# We'll search for launcher.config.json next to this script and one level up
def find_config():
    here = os.path.abspath(os.path.dirname(__file__))
    candidates = [
        os.path.join(here, "launcher.config.json"),
        os.path.join(here, "launcher", "launcher.config.json"),
        os.path.join(here, "..", "launcher.config.json"),
        os.path.join(here, "..", "launcher", "launcher.config.json"),
    ]
    for p in candidates:
        if os.path.isfile(p):
            return p
    # fallback to relative path used previously
    return os.path.join(here, "launcher.config.json")

CONFIG_PATH = find_config()

# AURA THEME
COLOR_BG = "#000000"
COLOR_FG = "#00FF00"  # Hacker Green
COLOR_BTN = "#1a1a1a"
COLOR_BTN_ACTIVE = "#333333"
FONT_MAIN = ("Consolas", 10)
FONT_HEADER = ("Consolas", 12, "bold")

class ServiceControl:
    def __init__(self, master, service_config, log_callback):
        self.config = service_config
        self.name = service_config.get('name', 'service')
        self.cmd = service_config.get('cmd')
        self.args = service_config.get('args', [])
        self.cwd = service_config.get('cwd', '.')
        self.log_callback = log_callback
        self.process = None
        # inferred port (from config url or explicit 'port')
        self.port = None
        # attempt to parse url field for port
        url = service_config.get('url')
        if url:
            try:
                up = urlparse(url)
                if up.port:
                    self.port = up.port
            except Exception:
                pass
        if not self.port:
            p = service_config.get('port')
            try:
                if p:
                    self.port = int(p)
            except Exception:
                self.port = None

        # Resolve CWD relative to script location if needed
        if not os.path.isabs(self.cwd):
            base = os.path.abspath(os.path.dirname(__file__))
            self.cwd = os.path.abspath(os.path.join(base, self.cwd))

        # UI Row
        self.frame = tk.Frame(master, bg=COLOR_BG, pady=5)
        self.frame.pack(fill='x', padx=10)

        # Left: Name
        self.lbl_name = tk.Label(self.frame, text=self.name.upper(), width=15, anchor='w', 
                     bg=COLOR_BG, fg=COLOR_FG, font=FONT_HEADER)
        self.lbl_name.pack(side='left')

        # Center: Info (one-line status)
        self.lbl_info = tk.Label(self.frame, text="stopped", anchor='w', bg=COLOR_BG, fg="#cccccc", font=FONT_MAIN)
        self.lbl_info.pack(side='left', expand=True, fill='x', padx=8)

        # Status Light
        self.canvas_status = tk.Canvas(self.frame, width=20, height=20, bg=COLOR_BG, highlightthickness=0)
        self.light = self.canvas_status.create_oval(5, 5, 15, 15, fill="gray")
        self.canvas_status.pack(side='left', padx=10)

        # Button
        # Buttons will emit intents if handlers are provided; default to direct start/stop
        self.request_start_fn = None
        self.request_stop_fn = None
        self.request_restart_fn = None
        self.request_force_stop_fn = None

        self.btn_action = tk.Button(self.frame, text="START", command=self.toggle_service,
                                    bg=COLOR_BTN, fg=COLOR_FG, font=FONT_MAIN,
                                    activebackground=COLOR_BTN_ACTIVE, activeforeground=COLOR_FG,
                                    bd=1, relief="flat", width=10)
        self.btn_action.pack(side='left', padx=10)

        # Kill occupant (enabled when port is occupied)
        self.occupant_pids = []
        self.btn_kill = tk.Button(self.frame, text="KILL PORT", command=self.kill_occupant,
                      bg="#444", fg="#FFF", font=FONT_MAIN, bd=1, relief="flat", width=12)
        self.btn_kill.pack(side='left', padx=8)
        self.btn_kill.config(state='disabled')
        # Force-start button (kills occupant(s) then starts)
        self.btn_force = tk.Button(self.frame, text="FORCE START", command=self.force_start,
                       bg="#550000", fg="#FFF", font=FONT_MAIN, bd=1, relief="flat", width=12)
        self.btn_force.pack(side='left', padx=8)
        self.btn_force.config(state='disabled')

        # Clear lock button (manual recovery)
        self.btn_clear = tk.Button(self.frame, text="CLEAR LOCK", command=self.clear_lock,
                       bg="#444444", fg="#FFF", font=FONT_MAIN, bd=1, relief="flat", width=12)
        self.btn_clear.pack(side='left', padx=8)
        self.btn_clear.config(state='disabled')

        # PID lock path
        safe_name = re.sub(r"[^0-9A-Za-z_.-]", "_", self.name)
        self.lock_file = os.path.join(tempfile.gettempdir(), f"aura_launcher_{safe_name}.lock")
        # preflight: check existing lock or port
        self.preflight_check()
        # callback to notify state machine of external state changes (optional)
        self.notify_state_fn = None

    def toggle_service(self):
        # If a request handler is registered, use the state-machine intent API
        if self.request_start_fn and self.request_stop_fn:
            # determine intent by current button text
            if self.btn_action['text'].upper().startswith('START'):
                try:
                    self.request_start_fn(self.name)
                except Exception as e:
                    self.log_callback(f"[{self.name}] Request start failed: {e}")
            else:
                try:
                    self.request_stop_fn(self.name)
                except Exception as e:
                    self.log_callback(f"[{self.name}] Request stop failed: {e}")
            return

        # fallback to direct control (legacy)
        if self.process is None:
            self.start()
        else:
            self.stop()

    def preflight_check(self):
        # If lock exists but PID is not alive, remove stale lock automatically
        if os.path.isfile(self.lock_file):
            if self.check_pid_lock():
                self.log_callback(f"[{self.name}] Existing PID lock detected and process is alive; start disabled until cleared.")
                self.btn_action.config(state='disabled')
                self.btn_clear.config(state='normal')
                return
            else:
                # stale lock
                try:
                    os.remove(self.lock_file)
                    self.log_callback(f"[{self.name}] Stale PID lock found and removed.")
                except Exception:
                    self.log_callback(f"[{self.name}] Failed to remove stale PID lock; use CLEAR LOCK.")
                    self.btn_clear.config(state='normal')
                    self.btn_action.config(state='disabled')
                    return
        # Check port occupancy
        if self.port:
            pids = self.find_pids_by_port(self.port)
            if pids:
                self.log_callback(f"[{self.name}] Port {self.port} in use by PID(s) {pids}; start disabled until cleared.")
                self.enable_kill_for_pids(pids)
                self.btn_action.config(state='disabled')
                self.btn_force.config(state='normal')
                self.btn_clear.config(state='normal')
                return
        # nothing blocking: ensure buttons enabled
        self.btn_action.config(state='normal')
        self.btn_clear.config(state='disabled')

    def start(self, force=False, env=None):
        if not self.cmd:
            self.log_callback(f"[{self.name}] No command configured.")
            return

        # PID lock: prevent double-start
        if self.check_pid_lock():
            self.log_callback(f"[{self.name}] PID lock present; service appears running. Aborting start.")
            return

        # Port preflight: if port occupied and not forcing, enable kill and bail
        if not force and self.port:
            pids = self.find_pids_by_port(self.port)
            if pids:
                self.log_callback(f"[{self.name}] Port {self.port} already in use by PID(s) {pids}. Use KILL PORT or FORCE START.")
                self.enable_kill_for_pids(pids)
                # disable start until user resolves
                self.btn_action.config(state='disabled')
                self.btn_force.config(state='normal')
                return

        try:
            full_cmd = [self.cmd] + self.args
            cmd_str = ' '.join(full_cmd)
            self.log_callback(f"[{self.name}] Starting: {cmd_str}")

            creationflags = 0
            if os.name == 'nt' and hasattr(subprocess, 'CREATE_NO_WINDOW'):
                creationflags = subprocess.CREATE_NO_WINDOW

            # Prepare environment
            # Always construct an explicit environment mapping so child processes
            # receive the expected variables on Windows. Merge provided env entries.
            env_map = os.environ.copy()
            if env is not None:
                for k, v in env.items():
                    env_map[str(k)] = str(v)

            # Use shell to allow running npm/cargo wrappers; pass a single string
            self.process = subprocess.Popen(
                cmd_str,
                cwd=self.cwd,
                shell=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                bufsize=1,
                universal_newlines=True,
                creationflags=creationflags,
                env=env_map
            )

            # Update UI
            self.canvas_status.itemconfig(self.light, fill="#00FF00")
            self.btn_action.config(text="STOP", bg="#330000", fg="#FF0000")
            # write PID lock
            try:
                with open(self.lock_file, 'w') as lf:
                    lf.write(str(self.process.pid))
            except Exception:
                pass

            # Start monitoring threads
            threading.Thread(target=self.monitor_output, args=(self.process.stdout, "OUT"), daemon=True).start()
            threading.Thread(target=self.monitor_output, args=(self.process.stderr, "ERR"), daemon=True).start()
            threading.Thread(target=self.monitor_process, daemon=True).start()

            # disable kill button while process is running
            self.btn_kill.config(state='disabled')
            self.btn_force.config(state='disabled')
            self.btn_action.config(state='normal')

        except Exception as e:
            self.log_callback(f"[{self.name}] Failed to start: {str(e)}")

        # no wrapper: direct return
        return

    def find_pids_by_port(self, port):
        """Return a list of PIDs listening on the given TCP port (Windows netstat)."""
        pids = set()
        try:
            if os.name == 'nt':
                out = subprocess.check_output(["netstat", "-ano"], universal_newlines=True, stderr=subprocess.DEVNULL)
                # lines like:  TCP    0.0.0.0:5175           0.0.0.0:0              LISTENING       1234
                for line in out.splitlines():
                    if f":{port} " in line or f":{port}\t" in line:
                        parts = re.split(r"\s+", line.strip())
                        if parts:
                            pid = parts[-1]
                            if pid.isdigit():
                                pids.add(int(pid))
            else:
                # Try lsof on Unix
                out = subprocess.check_output(["lsof", "-i", f":{port}"], universal_newlines=True, stderr=subprocess.DEVNULL)
                for line in out.splitlines()[1:]:
                    parts = re.split(r"\s+", line.strip())
                    if len(parts) >= 2 and parts[1].isdigit():
                        pids.add(int(parts[1]))
        except Exception:
            pass
        return list(pids)

    def check_pid_lock(self):
        """Return True if a lock exists and the PID is alive."""
        if not os.path.isfile(self.lock_file):
            return False
        try:
            with open(self.lock_file, 'r') as f:
                txt = f.read().strip()
            if not txt.isdigit():
                return False
            pid = int(txt)
            if pid <= 0:
                return False
            # check process existence
            if os.name == 'nt':
                out = subprocess.check_output(['tasklist', '/FI', f'PID eq {pid}'], universal_newlines=True, stderr=subprocess.DEVNULL)
                return str(pid) in out
            else:
                try:
                    os.kill(pid, 0)
                    return True
                except Exception:
                    return False
        except Exception:
            return False

    def remove_pid_lock(self):
        try:
            if os.path.isfile(self.lock_file):
                os.remove(self.lock_file)
        except Exception:
            pass

    def clear_lock(self):
        try:
            if os.path.isfile(self.lock_file):
                os.remove(self.lock_file)
                self.log_callback(f"[{self.name}] PID lock cleared by user.")
            else:
                self.log_callback(f"[{self.name}] No PID lock to clear.")
        except Exception as e:
            self.log_callback(f"[{self.name}] Failed to clear PID lock: {e}")
            return
        # re-run preflight to update UI
        self.preflight_check()

    def set_state(self, state, info=None):
        # Update visual state only; state is one of STOPPED/STARTING/HEALTHY/FAILED/STALE
        color_map = {
            'STOPPED': 'gray',
            'STARTING': 'yellow',
            'HEALTHY': '#00FF00',
            'FAILED': '#FF0000',
            'STALE': 'orange'
        }
        fill = color_map.get(state, 'gray')
        try:
            self.canvas_status.itemconfig(self.light, fill=fill)
        except Exception:
            pass
        # Update info text
        info_text = info or state.lower()
        try:
            self.lbl_info.config(text=info_text)
        except Exception:
            pass

    def force_start(self):
        # Attempt to kill occupants then start
        if self.occupant_pids:
            self.log_callback(f"[{self.name}] Force-start: killing occupant PID(s) {self.occupant_pids} before starting.")
            self.kill_occupant()
            # small wait to let OS release sockets
            threading.Timer(0.8, lambda: self.start(force=True)).start()
        else:
            self.start(force=True)

    def enable_kill_for_pids(self, pids):
        if not pids:
            return
        self.occupant_pids = pids
        if len(pids) == 1:
            self.btn_kill.config(text=f"KILL PID {pids[0]}")
        else:
            self.btn_kill.config(text=f"KILL PIDS ({len(pids)})")
        self.btn_kill.config(state='normal')

    def kill_occupant(self):
        if not self.occupant_pids:
            return
        for pid in self.occupant_pids:
            try:
                if pid and pid > 0:
                    if os.name == 'nt':
                        subprocess.run(f"taskkill /F /PID {pid}", shell=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
                    else:
                        os.kill(pid, signal.SIGTERM)
                else:
                    self.log_callback(f"[{self.name}] Killed occupant PID {pid}")
            except Exception as e:
                self.log_callback(f"[{self.name}] Failed killing PID {pid}: {e}")
        # disable button after attempt
        self.btn_kill.config(state='disabled')
        self.occupant_pids = []
        # re-enable start button after kill attempt
        self.btn_action.config(state='normal')
        self.btn_force.config(state='normal')
        # after kill, clear lock if present
        try:
            if os.path.isfile(self.lock_file):
                os.remove(self.lock_file)
                self.log_callback(f"[{self.name}] Removed PID lock after killing occupant(s).")
        except Exception:
            pass

    def stop(self):
        if self.process:
            try:
                pid = self.process.pid
                self.log_callback(f"[{self.name}] Stopping PID {pid}...")
                # FORCE KILL for Windows (Taskkill /T kills child processes)
                if os.name == 'nt':
                    subprocess.run(f"taskkill /F /T /PID {pid}", shell=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
                else:
                    try:
                        os.killpg(os.getpgid(pid), 15)
                    except Exception:
                        self.process.terminate()
            except Exception as e:
                self.log_callback(f"[{self.name}] Error stopping process: {e}")
            finally:
                self.process = None
                # remove PID lock
                try:
                    self.remove_pid_lock()
                except Exception:
                    pass

        # Reset UI
        self.canvas_status.itemconfig(self.light, fill="gray")
        self.btn_action.config(text="START", bg=COLOR_BTN, fg=COLOR_FG)

    def monitor_output(self, stream, label):
        """Reads stdout/stderr and sends to log window"""
        if not stream:
            return
        try:
            for line in iter(stream.readline, ''):
                if line:
                    txt = line.strip()
                    # Strip ANSI color codes which Vite may emit
                    clean_txt = re.sub(r'\x1B\[[0-?]*[ -/]*[@-~]', '', txt)
                    self.log_callback(f"[{self.name}] {clean_txt}")
                    # Detect Vite reported local URL: "Local: http://localhost:5173/" (also handle 127.0.0.1)
                    if self.name.lower() == 'vite':
                        m = re.search(r"Local:\s+https?://[^\s:]+:(\d+)", clean_txt, re.IGNORECASE)
                        if m:
                            try:
                                p = int(m.group(1))
                                if self.port != p:
                                    self.port = p
                                    self.log_callback(f"[{self.name}] Detected dev server port: {p}")
                                    # Notify state machine that vite is ready if hook present
                                    try:
                                        if getattr(self, 'notify_state_fn', None):
                                            # Construct canonical URL and mark HEALTHY immediately
                                            url = f"http://127.0.0.1:{p}/"
                                            self.notify_state_fn('HEALTHY', url)
                                    except Exception:
                                        pass
                            except Exception:
                                pass
                    # detect common bind/address-in-use messages
                    low = txt.lower()
                    if ('bind' in low and 'address already' in low) or ('only one usage of each socket address' in low) or ('address already in use' in low) or ('bind:' in low):
                        # try to find port from config 'port' or 'url', otherwise parse from message
                        port = self.port or self.config.get('port')
                        if not port:
                            m = re.search(r':(\d{2,5})', txt)
                            if m:
                                try:
                                    port = int(m.group(1))
                                except Exception:
                                    port = None
                        if port:
                            pids = self.find_pids_by_port(port)
                            if pids:
                                self.enable_kill_for_pids(pids)
                        else:
                            # fallback: scan a few common ports used by the project
                            common = [8080, 8443, 5173, 5174, 5175, 11434]
                            for cp in common:
                                pids = self.find_pids_by_port(cp)
                                if pids:
                                    self.enable_kill_for_pids(pids)
                                    break
        except Exception:
            pass
        finally:
            try:
                stream.close()
            except Exception:
                pass

    def monitor_process(self):
        """Watches for unexpected death"""
        if self.process:
            self.process.wait()
            if self.process: # If we didn't manually set it to None yet
                rc = self.process.returncode
                self.log_callback(f"[{self.name}] Process exited with code {rc}")
                # If exited with error and a port is configured, try to find occupant
                if rc != 0:
                    port = self.config.get('port')
                    if port:
                        pids = self.find_pids_by_port(port)
                        if pids:
                            self.enable_kill_for_pids(pids)
                # reset UI state
                self.canvas_status.itemconfig(self.light, fill="gray")
                self.btn_action.config(text="START", bg=COLOR_BTN, fg=COLOR_FG)
                # remove PID lock
                try:
                    self.remove_pid_lock()
                except Exception:
                    pass
                self.process = None


class AuraLauncher:
    def __init__(self, root):
        self.root = root
        self.root.title("AURA LAUNCHER // CONTROL PLANE")
        self.root.geometry("900x600")
        self.root.configure(bg=COLOR_BG)
        # Top: Header + global controls
        header = tk.Label(root, text="AURA Launcher", bg=COLOR_BG, fg=COLOR_FG, font=("Consolas", 18, "bold"), pady=8)
        header.pack(side="top")

        controls_frame = tk.Frame(root, bg=COLOR_BG)
        controls_frame.pack(side='top', pady=6)

        self.btn_start_all = tk.Button(controls_frame, text="START ALL (Sequenced)", command=self.start_all,
                                       bg="#005500", fg="#FFFFFF", font=FONT_MAIN, width=20)
        self.btn_start_all.pack(side='left', padx=8)

        self.btn_stop_all = tk.Button(controls_frame, text="STOP ALL", command=self.stop_all,
                                      bg="#333333", fg="#FFFFFF", font=FONT_MAIN, width=12)
        self.btn_stop_all.pack(side='left', padx=8)

        self.btn_emerg = tk.Button(controls_frame, text="Emergency Kill", command=self.emergency_kill,
                                   bg="#770000", fg="#FFFFFF", font=FONT_MAIN, width=14)
        self.btn_emerg.pack(side='left', padx=8)

        # Middle: Services Container (spine)
        self.service_container = tk.Frame(root, bg=COLOR_BG)
        self.service_container.pack(fill='x', pady=6)

        # Bottom: Logs and diagnostics (simple append-only log for now)
        self.log_area = scrolledtext.ScrolledText(root, bg="#111", fg="#ccc", font=("Consolas", 9), state='disabled', height=12)
        self.log_area.pack(fill='both', expand=True, padx=10, pady=10)

        # state machine
        # Import the state_machine module from the local file without relying on package imports
        import importlib.util
        mod_path = os.path.join(os.path.dirname(__file__), 'state_machine.py')
        spec = importlib.util.spec_from_file_location('launcher.state_machine', mod_path)
        state_machine = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(state_machine)
        StateMachine = state_machine.StateMachine
        # keep module reference and BootSequencer class on self for later use
        self.state_machine_module = state_machine
        self.BootSequencerClass = getattr(state_machine, 'BootSequencer', None)
        self.sm = StateMachine(self.log_message)

        # Load Config and wire services
        self.load_services()

    def load_services(self):
        try:
            with open(CONFIG_PATH, 'r') as f:
                config = json.load(f)
            services = config.get('services', [])
            if not services:
                self.log_message("No services defined in config.")
            # canonical spine order: ollama -> backend -> vite -> electron (IDs are lowercase)
            spine = ['ollama', 'backend', 'vite', 'electron']
            # create mapping name->config (lowercased)
            cfg_map = {s.get('name', '').lower(): s for s in services}

            def find_config_for(canonical):
                # map canonical ids to expected config names/aliases
                aliases = {
                    'ollama': ['ollama'],
                    'backend': ['backend'],
                    'vite': ['vite'],
                    'electron': ['electron']
                }
                for a in aliases.get(canonical, [canonical]):
                    c = cfg_map.get(a)
                    if c:
                        return c
                return None

            created = []
            for canonical in spine:
                c = find_config_for(canonical)
                if not c:
                    # create a stub so the UI shows the row but the service has no start command
                    c = {'name': canonical, 'cmd': None, 'args': [], 'cwd': '.'}
                sc = ServiceControl(self.service_container, c, self.log_message)
                # register request handlers so UI buttons emit intents (capture canonical id)
                sc.request_start_fn = (lambda cid=canonical: (lambda name=cid: self.sm.request_start(name)))()
                sc.request_stop_fn = (lambda cid=canonical: (lambda name=cid: self.sm.request_stop(name)))()
                sc.request_force_stop_fn = (lambda cid=canonical: (lambda name=cid: self.sm.request_force_stop(name)))()
                # wire notify_state_fn so ServiceControl can tell state machine about immediate events (e.g., vite port detected)
                sc.notify_state_fn = (lambda svc=canonical: (lambda state, info=None: self.sm.set_state(svc, state, info)))()
                # register service with state machine; dependencies follow spine order
                deps = []
                if canonical == 'backend':
                    deps = ['ollama']
                if canonical == 'vite':
                    deps = ['backend']
                if canonical == 'electron':
                    deps = ['vite']
                self.sm.register(canonical, sc, deps=deps)
                
            self.log_message("System initialized. Ready.")
            self.log_message(f"Config: {CONFIG_PATH}")
            # instantiate boot sequencer (reactive) after services registered
            BootSequencer = getattr(self, 'BootSequencerClass', None)
            if BootSequencer:
                self.sequencer = BootSequencer(self.sm, ['ollama', 'backend', 'vite', 'electron'], self.log_message)
            else:
                self.sequencer = None
            
        except FileNotFoundError:
            self.log_message(f"ERROR: Could not find config at {CONFIG_PATH}")
        except Exception as e:
            self.log_message(f"ERROR: {str(e)}")

    def log_message(self, message):
        def _log():
            self.log_area.config(state='normal')
            self.log_area.insert(tk.END, message + "\n")
            self.log_area.see(tk.END)
            self.log_area.config(state='disabled')
        self.root.after(0, _log)

    # Top-level controls mapped to state machine
    def start_all(self):
        # Use reactive boot sequencer if available
        if getattr(self, 'sequencer', None):
            self.sequencer.start()
            return
        order = ['ollama', 'backend', 'vite', 'electron']
        threading.Thread(target=lambda: self.sm.request_start_all(order), daemon=True).start()

    def stop_all(self):
        # stop in reverse order to be safe
        for name in reversed(['Electron', 'Vite', 'Backend', 'Ollama']):
            try:
                self.sm.request_stop(name)
            except Exception:
                pass

    def emergency_kill(self):
        # brute force: force-stop every registered service
        for k in list(self.sm.services.keys()):
            try:
                self.sm.request_force_stop(k)
            except Exception as e:
                self.log_message(f"[emergency] failed to force-stop {k}: {e}")


if __name__ == "__main__":
    root = tk.Tk()
    app = AuraLauncher(root)
    root.mainloop()
