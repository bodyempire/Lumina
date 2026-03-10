"""
lumina_py.py — Python bindings for the Lumina runtime via ctypes.
Usage:
    from lumina_py import LuminaRuntime
    rt = LuminaRuntime.from_source(source_code)
    rt.apply_event("moto1", "battery", 18)
    print(rt.export_state())
"""
import ctypes
import json
import os
import sys


def _load_library():
    if sys.platform == "darwin":
        name = "liblumina_ffi.dylib"
    elif sys.platform == "win32":
        name = "lumina_ffi.dll"
    else:
        name = "liblumina_ffi.so"

    search = [
        os.path.join(os.path.dirname(__file__), "..", "..", "target", "release", name),
        os.path.join(os.path.dirname(__file__), "..", "..", "target", "debug", name),
    ]
    for path in search:
        if os.path.exists(path):
            return ctypes.CDLL(os.path.abspath(path))
    raise FileNotFoundError(
        f"Could not find {name}. Run 'cargo build --release -p lumina-ffi' first."
    )


_lib = _load_library()

_lib.lumina_create.argtypes      = [ctypes.c_char_p]
_lib.lumina_create.restype       = ctypes.c_void_p

_lib.lumina_apply_event.argtypes = [ctypes.c_void_p, ctypes.c_char_p,
                                     ctypes.c_char_p, ctypes.c_char_p]
_lib.lumina_apply_event.restype  = ctypes.c_char_p

_lib.lumina_export_state.argtypes = [ctypes.c_void_p]
_lib.lumina_export_state.restype  = ctypes.c_char_p

_lib.lumina_tick.argtypes        = [ctypes.c_void_p]
_lib.lumina_tick.restype         = ctypes.c_char_p

_lib.lumina_last_error.argtypes  = [ctypes.c_void_p]
_lib.lumina_last_error.restype   = ctypes.c_char_p

_lib.lumina_free_string.argtypes = [ctypes.c_char_p]
_lib.lumina_free_string.restype  = None

_lib.lumina_destroy.argtypes     = [ctypes.c_void_p]
_lib.lumina_destroy.restype      = None


class LuminaRuntime:
    def __init__(self, handle):
        self._handle = handle

    @classmethod
    def from_source(cls, source: str) -> "LuminaRuntime":
        handle = _lib.lumina_create(source.encode("utf-8"))
        if handle is None:
            raise ValueError("Failed to create Lumina runtime — check source for errors")
        return cls(handle)

    @classmethod
    def from_file(cls, path: str) -> "LuminaRuntime":
        with open(path, "r") as f:
            return cls.from_source(f.read())

    def apply_event(self, instance: str, field: str, value) -> dict:
        value_json = json.dumps(value).encode("utf-8")
        raw = _lib.lumina_apply_event(
            self._handle,
            instance.encode("utf-8"),
            field.encode("utf-8"),
            value_json,
        )
        if raw is None:
            raise RuntimeError("lumina_apply_event returned null")
        result = raw.decode("utf-8")
        if result.startswith("ERROR:"):
            diag = json.loads(result[6:])
            raise RuntimeError(f"Lumina rollback: {diag['message']}\nFix: {diag['suggested_fix']}")
        return json.loads(result)

    def export_state(self) -> dict:
        raw = _lib.lumina_export_state(self._handle)
        if raw is None:
            raise RuntimeError("lumina_export_state returned null")
        return json.loads(raw.decode("utf-8"))

    def tick(self) -> list:
        raw = _lib.lumina_tick(self._handle)
        if raw is None:
            return []
        result = raw.decode("utf-8")
        if result.startswith("ERROR:"):
            diag = json.loads(result[6:])
            raise RuntimeError(f"Lumina tick rollback: {diag['message']}")
        return json.loads(result)

    def __del__(self):
        if self._handle:
            _lib.lumina_destroy(self._handle)
            self._handle = None
