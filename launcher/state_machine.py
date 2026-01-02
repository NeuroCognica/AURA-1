import threading
import time
import socket

STOPPED = 'STOPPED'
STARTING = 'STARTING'
HEALTHY = 'HEALTHY'
FAILED = 'FAILED'
STALE = 'STALE'

class StateMachine:
    def __init__(self, log_fn):
        self.log = log_fn
        self.services = {}  # name(lower) -> {svc, state, deps, meta}
        self.lock = threading.Lock()
        self.subscribers = []

    def register(self, name, svc_obj, deps=None):
        key = name.lower()
        self.services[key] = {'svc': svc_obj, 'state': STOPPED, 'deps': [d.lower() for d in (deps or [])], 'meta': {}}
        self.log(f"[state] Registered service '{name}' with deps={deps}")

    def get_state(self, name):
        return self.services.get(name.lower(), {}).get('state')

    def set_state(self, name, state, info=None):
        key = name.lower()
        with self.lock:
            if key not in self.services:
                return
            # update state
            self.services[key]['state'] = state
            # store info into meta if provided and looks like a URL
            if info is not None:
                try:
                    if isinstance(info, str) and info.startswith('http'):
                        self.services[key].setdefault('meta', {})['vite_url'] = info
                    else:
                        # generic info slot
                        self.services[key].setdefault('meta', {})['info'] = info
                except Exception:
                    pass
        svc = self.services[key]['svc']
        try:
            svc.set_state(state, info)
        except Exception:
            pass
        self.log(f"[state] {name} -> {state} {('('+info+')') if info else ''}")
        # notify subscribers
        for cb in list(self.subscribers):
            try:
                cb(name.lower(), state)
            except Exception:
                pass

    def subscribe(self, callback):
        """Callback signature: fn(service_name_lower, new_state)"""
        self.subscribers.append(callback)

    def _check_tcp(self, host, port, timeout=1.0):
        try:
            with socket.create_connection((host, port), timeout=timeout):
                return True
        except Exception:
            return False

    def _health_check(self, name):
        ent = self.services.get(name.lower())
        if not ent:
            return False
        svc = ent['svc']
        svc_name = getattr(svc, 'name', '').lower()
        # Ollama: TCP port (11434) -> ensure listening
        if svc_name == 'ollama' and getattr(svc, 'port', None):
            # Prefer HTTP health check for Ollama's API if available
            try:
                import http.client
                port = svc.port
                conn = http.client.HTTPConnection('127.0.0.1', port, timeout=2)
                conn.request('GET', '/api/tags')
                resp = conn.getresponse()
                ok = resp.status == 200
                try:
                    conn.close()
                except Exception:
                    pass
                return ok
            except Exception:
                # fallback to plain TCP
                return self._check_tcp('127.0.0.1', svc.port, timeout=1.0)

        # Backend: HTTP health endpoint /health
        if svc_name == 'backend':
            try:
                import http.client
                parsed = None
                # try to get url from config
                cfg = getattr(svc, 'config', None)
                url = None
                if cfg:
                    url = cfg.get('health', {}).get('url') if isinstance(cfg.get('health', {}), dict) else None
                if url:
                    from urllib.parse import urlparse
                    parsed = urlparse(url)
                host = '127.0.0.1'
                port = 8080
                path = '/health'
                if parsed:
                    host = parsed.hostname or host
                    port = parsed.port or port
                    path = parsed.path or path
                conn = http.client.HTTPConnection(host, port, timeout=2)
                conn.request('GET', path)
                resp = conn.getresponse()
                ok = resp.status == 200
                try:
                    conn.close()
                except Exception:
                    pass
                return ok
            except Exception:
                return False

        # Vite: require known port and HTTP 200 at '/'
        if svc_name == 'vite':
            if getattr(svc, 'port', None):
                try:
                    import http.client
                    conn = http.client.HTTPConnection('127.0.0.1', svc.port, timeout=2)
                    conn.request('GET', '/')
                    resp = conn.getresponse()
                    ok = resp.status == 200
                    try:
                        conn.close()
                    except Exception:
                        pass
                    return ok
                except Exception:
                    return False
            return False

        # Electron: process alive
        if svc_name == 'electron':
            try:
                return svc.check_pid_lock()
            except Exception:
                return False

        # Generic fallback: if port attribute exists, check TCP
        if getattr(svc, 'port', None):
            return self._check_tcp('127.0.0.1', svc.port, timeout=1.0)

        # fallback: PID lock
        try:
            if svc.check_pid_lock():
                return True
        except Exception:
            pass
        return False

    def request_start(self, name, force=False, env=None):
        key = name.lower()
        if key not in self.services:
            self.log(f"[state] request_start: unknown service {name}")
            return False
        # check deps
        deps = self.services[key]['deps']
        for d in deps:
            st = self.services.get(d, {}).get('state')
            if st != HEALTHY:
                self.log(f"[state] cannot start {name}: dependency {d} not healthy ({st})")
                return False

        svc = self.services[key]['svc']
        # spawn in thread
        def _start_thread():
            try:
                self.set_state(name, STARTING)
                # pass env through to service start if provided
                try:
                    if env is not None:
                        svc.start(force=force, env=env)
                    else:
                        svc.start(force=force)
                except TypeError:
                    # older ServiceControl may not accept env param
                    svc.start(force=force)
            except Exception as e:
                self.log(f"[state] spawn error {name}: {e}")
                self.set_state(name, FAILED, info=str(e))
                return

            # poll health for a bounded time (longer for vite)
            timeout = 30.0
            if key := name.lower():
                if key == 'vite':
                    timeout = 120.0
            start = time.time()
            while time.time() - start < timeout:
                if self._health_check(name):
                    self.set_state(name, HEALTHY)
                    return
                time.sleep(0.5)
            # timed out
            self.set_state(name, FAILED, info='health-timeout')

        threading.Thread(target=_start_thread, daemon=True).start()
        return True

    def request_stop(self, name):
        key = name.lower()
        if key not in self.services:
            self.log(f"[state] request_stop: unknown {name}")
            return False
        svc = self.services[key]['svc']
        try:
            svc.stop()
            self.set_state(name, STOPPED)
            return True
        except Exception as e:
            self.log(f"[state] stop error {name}: {e}")
            return False

    def request_force_stop(self, name):
        key = name.lower()
        if key not in self.services:
            return False
        svc = self.services[key]['svc']
        try:
            # best-effort: kill occupant(s) and stop
            if getattr(svc, 'occupant_pids', None):
                svc.kill_occupant()
            svc.stop()
            self.set_state(name, STOPPED)
            return True
        except Exception as e:
            self.log(f"[state] force-stop error {name}: {e}")
            return False

    def request_start_all(self, ordered_names):
        # Strict sequencing: start in order provided; stop on failure
        for name in ordered_names:
            self.log(f"[state] StartAll: starting {name}")
            ok = self.request_start(name)
            if not ok:
                self.log(f"[state] StartAll: failed to spawn {name}")
                return False
            # wait until healthy or failed
            start = time.time()
            while True:
                st = self.get_state(name)
                if st == HEALTHY:
                    break
                if st == FAILED:
                    self.log(f"[state] StartAll: {name} failed to become healthy")
                    return False
                if time.time() - start > 60:
                    self.log(f"[state] StartAll: timeout waiting for {name}")
                    return False
                time.sleep(0.5)
        return True


class BootSequencer:
    IDLE = 'IDLE'
    BOOTING = 'BOOTING'
    FAILED = 'FAILED'
    COMPLETE = 'COMPLETE'

    def __init__(self, state_machine, boot_order, log_fn):
        self.sm = state_machine
        self.boot_order = [s.lower() for s in boot_order]
        self.log = log_fn
        self.state = self.IDLE
        self.current_index = 0
        # subscribe to service changes
        self.sm.subscribe(self.on_service_change)

    def start(self):
        if self.state == self.BOOTING:
            return
        self.log('[sequencer] Start requested')
        self.state = self.BOOTING
        self.current_index = 0
        self.try_start_current()

    def try_start_current(self):
        if self.state != self.BOOTING:
            return
        if self.current_index >= len(self.boot_order):
            self.state = self.COMPLETE
            self.log('[sequencer] Boot complete')
            return
        svc = self.boot_order[self.current_index]
        st = self.sm.get_state(svc)
        self.log(f'[sequencer] evaluating {svc} -> {st}')
        if st == HEALTHY:
            self.advance()
            return
        if st == STOPPED:
            # For electron, ensure the vite_url actually responds over HTTP
            env = None
            if svc == 'electron':
                vite_entry = self.sm.services.get('vite')
                if vite_entry:
                    meta = vite_entry.get('meta', {})
                    url = meta.get('vite_url')
                    if url:
                        # wait up to 2s for the frontend to respond to HTTP GET /
                        ok = False
                        try:
                            import http.client
                            from urllib.parse import urlparse
                            parsed = urlparse(url)
                            host = parsed.hostname or 'localhost'
                            port = parsed.port or 80
                            path = parsed.path or '/'
                            start = time.time()
                            while time.time() - start < 2.0:
                                try:
                                    conn = http.client.HTTPConnection(host, port, timeout=2)
                                    conn.request('GET', path)
                                    resp = conn.getresponse()
                                    if resp.status == 200:
                                        ok = True
                                        try:
                                            conn.close()
                                        except Exception:
                                            pass
                                        break
                                    try:
                                        conn.close()
                                    except Exception:
                                        pass
                                except Exception:
                                    pass
                                time.sleep(0.25)
                        except Exception:
                            ok = False
                        if ok:
                            env = {'VITE_DEV_SERVER_URL': url, 'NODE_ENV': 'development'}
                        else:
                            # If the frontend didn't respond in time, still pass the URL
                            env = {'VITE_DEV_SERVER_URL': url, 'NODE_ENV': 'development'}
            # Log attempt and result for debugging why a service might not actually start
            try:
                self.log(f'[sequencer] attempting start {svc} with env={env}')
                res = self.sm.request_start(svc, env=env)
                self.log(f'[sequencer] request_start returned {res} for {svc}')
            except Exception as e:
                self.log(f'[sequencer] request_start raised for {svc}: {e}')
            return
        if st == STARTING:
            return
        if st in (FAILED, STALE):
            self.fail(svc)

    def advance(self):
        self.current_index += 1
        if self.current_index >= len(self.boot_order):
            self.state = self.COMPLETE
            self.log('[sequencer] Boot complete')
            return
        self.try_start_current()

    def fail(self, svc):
        self.state = self.FAILED
        self.log(f'[sequencer] Boot failed at {svc}')

    def on_service_change(self, svc_name, new_state):
        # only react when booting
        if self.state != self.BOOTING:
            return
        # if the changed service is at or before current_index, re-evaluate
        try:
            idx = self.boot_order.index(svc_name)
        except ValueError:
            return
        if idx > self.current_index:
            return
        # re-evaluate current
        self.try_start_current()
