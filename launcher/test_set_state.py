import importlib.util, os, sys
spec = importlib.util.spec_from_file_location('state_machine', os.path.abspath('state_machine.py'))
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)
SM = mod.StateMachine
# Create simple logger that prints
sm = SM(lambda m: print(m))
# register a dummy service object (minimal)
class Dummy:
    def __init__(self):
        self.name='vite'
        self.port=None
    def set_state(self,s,i=None):
        print('Dummy.set_state called', s, i)

sm.register('vite', Dummy(), deps=[])
# Call set_state with URL info
sm.set_state('vite', 'HEALTHY', 'http://127.0.0.1:5175/')
print('meta after set_state:', sm.services['vite'].get('meta'))
