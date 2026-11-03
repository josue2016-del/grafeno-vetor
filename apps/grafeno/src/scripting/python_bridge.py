# Grafeno Vetor – Python Bridge
# This module is injected into every Python script run inside Grafeno Vetor.
# It exposes a ``grafeno`` namespace with drawing primitives.
#
# IMPORTANT: this bridge works by sending JSON commands to Grafeno's
# internal HTTP/RPC interface (port configured in settings).  If no
# connection is available, drawing calls are logged and ignored.

import json
import sys
import os

# ---------------------------------------------------------------------------
# Connection to Grafeno (optional – works standalone too)
# ---------------------------------------------------------------------------

_GRAFENO_RPC_PORT = int(os.environ.get("GRAFENO_RPC_PORT", "0"))
_GRAFENO_TOKEN = os.environ.get("GRAFENO_RPC_TOKEN", "")

def _rpc(method, params=None):
    """Send an RPC call to Grafeno and return the parsed response."""
    if _GRAFENO_RPC_PORT == 0:
        return None
    try:
        import urllib.request
        import urllib.error
        url = f"http://127.0.0.1:{_GRAFENO_RPC_PORT}/rpc"
        body = json.dumps({"method": method, "params": params or {}}).encode()
        req = urllib.request.Request(url, data=body, headers={
            "Content-Type": "application/json",
            "Authorization": f"Bearer {_GRAFENO_TOKEN}" if _GRAFENO_TOKEN else "",
        })
        with urllib.request.urlopen(req, timeout=2) as resp:
            return json.loads(resp.read())
    except Exception as e:
        print(f"[grafeno bridge] RPC error: {e}", file=sys.stderr)
        return None

# ---------------------------------------------------------------------------
# Drawing API
# ---------------------------------------------------------------------------

class _ShapeRef:
    """Thin wrapper around a shape ID returned by Grafeno."""
    def __init__(self, shape_id):
        self.id = shape_id
    def __repr__(self):
        return f"<Shape {self.id}>"

class grafeno:
    """Grafeno Vetor drawing API for Python macros."""

    # -- document / page / layer ------------------------------------------

    @staticmethod
    def active_document():
        """Return info about the active document."""
        r = _rpc("vg_doc", {})
        return r or {}

    @staticmethod
    def active_page():
        doc = grafeno.active_document()
        return doc.get("page", 0)

    @staticmethod
    def active_layer():
        doc = grafeno.active_document()
        return doc.get("layer", 0)

    @staticmethod
    def get_layer(layer_id):
        return layer_id

    @staticmethod
    def all_layers():
        doc = grafeno.active_document()
        return doc.get("layers", [])

    # -- shapes -----------------------------------------------------------

    @staticmethod
    def rect(x, y, w, h, fill=None, outline=None):
        """Create a rectangle at (x, y) with size (w, h). Units = mm."""
        params = {"x": x, "y": y, "width": w, "height": h}
        if fill:
            params["fill"] = fill
        if outline:
            params["outline"] = outline
        r = _rpc("vg_create_rect", params)
        return _ShapeRef(r["id"]) if r else None

    @staticmethod
    def ellipse(cx, cy, rx, ry, fill=None, outline=None):
        """Create an ellipse centred at (cx, cy) with radii (rx, ry)."""
        params = {"cx": cx, "cy": cy, "rx": rx, "ry": ry}
        if fill:
            params["fill"] = fill
        if outline:
            params["outline"] = outline
        r = _rpc("vg_create_ellipse", params)
        return _ShapeRef(r["id"]) if r else None

    @staticmethod
    def text(x, y, content, font="Arial", size=12, bold=False):
        """Create artistic text at (x, y)."""
        params = {"x": x, "y": y, "text": content, "font": font, "size_pt": size}
        if bold:
            params["bold"] = bold
        r = _rpc("vg_create_text", params)
        return _ShapeRef(r["id"]) if r else None

    @staticmethod
    def group(shape_ids):
        """Group a list of shape IDs into a single group."""
        r = _rpc("vg_group", {"ids": shape_ids})
        return _ShapeRef(r["id"]) if r else None

    @staticmethod
    def delete(shape_id):
        _rpc("vg_delete", {"id": shape_id})

    @staticmethod
    def select(shape_id):
        _rpc("vg_select", {"id": shape_id})

    @staticmethod
    def selection():
        doc = grafeno.active_document()
        return doc.get("selection", [])

    # -- transforms -------------------------------------------------------

    @staticmethod
    def move(shape_id, dx, dy):
        _rpc("vg_transform", {"id": shape_id, "dx": dx, "dy": dy})

    @staticmethod
    def scale(shape_id, sx, sy, cx=None, cy=None):
        params = {"id": shape_id, "sx": sx, "sy": sy}
        if cx is not None:
            params["cx"] = cx
        if cy is not None:
            params["cy"] = cy
        _rpc("vg_transform_scale", params)

    @staticmethod
    def rotate(shape_id, angle, cx=None, cy=None):
        params = {"id": shape_id, "angle": angle}
        if cx is not None:
            params["cx"] = cx
        if cy is not None:
            params["cy"] = cy
        _rpc("vg_transform_rotate", params)

    # -- colours ----------------------------------------------------------

    @staticmethod
    def set_fill(shape_id, r, g, b, a=1.0):
        """Set uniform fill colour (0–1 range)."""
        _rpc("vg_set_fill", {
            "id": shape_id,
            "color": {"r": r, "g": g, "b": b, "a": a},
        })

    @staticmethod
    def set_outline(shape_id, width, r, g, b):
        _rpc("vg_set_outline", {
            "id": shape_id,
            "width": width,
            "color": {"r": r, "g": g, "b": b},
        })

    # -- page info --------------------------------------------------------

    @staticmethod
    def page_size():
        doc = grafeno.active_document()
        return doc.get("page_size", (0, 0))

    @staticmethod
    def unit():
        doc = grafeno.active_document()
        return doc.get("units", "mm")

    # -- helpers ----------------------------------------------------------

    @staticmethod
    def count_shapes():
        doc = grafeno.active_document()
        return doc.get("shape_count", 0)

    @staticmethod
    def comment(msg):
        print(f"[grafeno] {msg}")


# ---------------------------------------------------------------------------
# Convenience: run() hook
# ---------------------------------------------------------------------------

def run():
    """Entry point — override this in your macro."""
    pass
