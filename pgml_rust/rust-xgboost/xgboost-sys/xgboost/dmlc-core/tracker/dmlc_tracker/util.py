import sys

# Compatibility shim for handling strings
PY3 = (sys.version_info[0] == 3)

if PY3:
    def py_str(x):
        """convert c string back to python string"""
        return x.decode('utf-8')
else:
    def py_str(x):
        """convert c string back to python string"""
        return x
