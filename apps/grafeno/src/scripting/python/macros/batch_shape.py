# Grafeno Vetor – Batch Shape Generator Macro
# Creates multiple copies of a shape in a grid or radial pattern.
#
# Usage:
#   from batch_shape import grid_shapes, radial_shapes
#   grid_shapes(kind="rect", count=12, cols=4, spacing=20)
#   radial_shapes(kind="ellipse", count=8, radius=30)

import math

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _rect(x, y, w, h, fill=None, outline=None):
    return grafeno.rect(x, y, w, h, fill=fill, outline=outline)

def _ellipse(cx, cy, rx, ry, fill=None, outline=None):
    return grafeno.ellipse(cx, cy, rx, ry, fill=fill, outline=outline)

def _text(x, y, content, font="Arial", size=12):
    return grafeno.text(x, y, content, font=font, size=size)

def _group(ids):
    return grafeno.group(ids) if ids else None


# ---------------------------------------------------------------------------
# Grid pattern
# ---------------------------------------------------------------------------

def grid_shapes(kind="rect", count=9, cols=3, spacing=20,
                size=15, fill_col=(0.2, 0.5, 0.9), outline_col=(0.0, 0.0, 0.0)):
    """
    Create `count` shapes in a grid.

    Parameters
    ----------
    kind : str
        "rect" or "ellipse"
    count : int
        Total number of shapes.
    cols : int
        Number of columns.
    spacing : float
        Gap between shapes (mm).
    size : float
        Width/height of each shape (mm).
    fill_col : tuple
        RGB fill colour (0–1).
    outline_col : tuple
        RGB outline colour (0–1).
    """
    rows = math.ceil(count / cols)
    shapes = []
    start_x = 20.0
    start_y = 20.0 + rows * (size + spacing)

    for i in range(count):
        col = i % cols
        row = i // cols
        x = start_x + col * (size + spacing)
        y = start_y - row * (size + spacing)

        if kind == "rect":
            s = _rect(x, y, size, size, fill=fill_col, outline=outline_col)
        elif kind == "ellipse":
            s = _ellipse(x + size / 2, y + size / 2,
                         size / 2, size / 2,
                         fill=fill_col, outline=outline_col)
        else:
            grafeno.comment(f"Unknown kind: {kind}")
            continue
        if s:
            shapes.append(s)

    if shapes:
        _group([sh.id for sh in shapes])
    grafeno.comment(f"Created {len(shapes)} shapes in {rows}×{cols} grid")
    return shapes


# ---------------------------------------------------------------------------
# Radial pattern
# ---------------------------------------------------------------------------

def radial_shapes(kind="ellipse", count=8, radius=40, size=12,
                  center_x=100, center_y=100,
                  fill_col=(0.9, 0.4, 0.2), outline_col=(0.0, 0.0, 0.0)):
    """
    Create `count` shapes arranged in a circle.
    """
    shapes = []
    angle_step = 2 * math.pi / count

    for i in range(count):
        angle = i * angle_step
        sx = center_x + radius * math.cos(angle)
        sy = center_y + radius * math.sin(angle)

        if kind == "ellipse":
            s = _ellipse(sx, sy, size / 2, size / 2,
                         fill=fill_col, outline=outline_col)
        elif kind == "rect":
            s = _rect(sx - size / 2, sy - size / 2, size, size,
                      fill=fill_col, outline=outline_col)
        else:
            continue
        if s:
            shapes.append(s)

    if shapes:
        _group([sh.id for sh in shapes])
    grafeno.comment(f"Created {len(shapes)} shapes in radial pattern")
    return shapes


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

def run():
    """Run a demo: 12 rectangles in a 4×3 grid."""
    grid_shapes(kind="rect", count=12, cols=4, spacing=18, size=14)
