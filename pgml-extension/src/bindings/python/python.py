#
# Activate and use virtualenv
#
import os
import sys

__venv = None

def activate_venv(venv):
    global __venv
    if __venv == venv:
        return True

    if sys.platform in ('win32', 'win64', 'cygwin'):
        activate_this = os.path.join(venv, 'Scripts', 'activate_this.py')
    else:
        activate_this = os.path.join(venv, 'bin', 'activate_this.py')

    if os.path.exists(activate_this):
        exec(open(activate_this).read(), dict(__file__=activate_this))
        __venv = venv
        return True
    else:
        raise RuntimeError("Virtual environment '{}' does not exist".format(venv))


def freeze():
    try:
        from pip._internal.operations import freeze
    except ImportError: # pip < 10.0
        from pip.operations import freeze

    return list(freeze.freeze())


def cuda_available():
    import torch
    return torch.cuda.is_available()
