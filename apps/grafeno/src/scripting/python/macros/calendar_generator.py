# Grafeno Vetor – Calendar Generator Macro
# Generates a full-year or single-month calendar as vector shapes.
#
# Usage:
#   from calendar_generator import generate_calendar
#   generate_calendar(year=2026, month=1)          # single month
#   generate_calendar(year=2026, layout="year")   # full year grid
#
# The macro creates rectangles for days, text for labels, and groups
# everything by month.  All coordinates are in millimetres.

import math
import calendar
import datetime

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

PAGE_WIDTH_MM = 210.0   # A4 width
PAGE_HEIGHT_MM = 297.0

DAY_WIDTH = 14.0        # mm per day cell
DAY_HEIGHT = 12.0
MARGIN_LEFT = 20.0
MARGIN_TOP = 30.0
MONTH_GAP_X = 5.0
MONTH_GAP_Y = 5.0

FONT_NAME = "Arial"
FONT_SIZE_DAY = 8.0
FONT_SIZE_MONTH = 14.0
FONT_SIZE_YEAR = 20.0

DAYS_OF_WEEK = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]

# Colour palette
HEADER_COLOUR = (0.1, 0.3, 0.6)    # RGB 0–1
WEEKDAY_COLOUR = (0.2, 0.2, 0.2)
DAY_COLOUR = (0.9, 0.9, 0.9)
WEEKEND_COLOUR = (0.95, 0.85, 0.85)
TEXT_COLOUR = (0.0, 0.0, 0.0)
GRID_COLOUR = (0.5, 0.5, 0.5)


# ---------------------------------------------------------------------------
# Helpers (call into grafeno bridge)
# ---------------------------------------------------------------------------

def _rect(x, y, w, h, fill=None, outline=None):
    return grafeno.rect(x, y, w, h, fill=fill, outline=outline)

def _text(x, y, content, font=FONT_NAME, size=FONT_SIZE_DAY, bold=False):
    return grafeno.text(x, y, content, font=font, size=size, bold=bold)

def _group(ids):
    return grafeno.group(ids) if ids else None

def _set_fill(shape, r, g, b, a=1.0):
    if shape:
        grafeno.set_fill(shape.id, r, g, b, a)

def _set_outline(shape, width, r, g, b):
    if shape:
        grafeno.set_outline(shape.id, width, r, g, b)


# ---------------------------------------------------------------------------
# Single month calendar
# ---------------------------------------------------------------------------

def build_month_grid(year, month, origin_x, origin_y):
    """Build a single month grid. Returns list of created shapes."""
    all_shapes = []

    # Month title
    month_name = calendar.month_name[month]
    title = _text(origin_x, origin_y + DAY_HEIGHT * 1.5,
                  f"{month_name} {year}",
                  font=FONT_NAME, size=FONT_SIZE_MONTH, bold=True)
    all_shapes.append(title)
    grafeno.set_fill(title.id, HEADER_COLOUR[0], HEADER_COLOUR[1], HEADER_COLOUR[2])

    # Weekday headers
    for i, dw in enumerate(DAYS_OF_WEEK):
        x = origin_x + i * DAY_WIDTH
        cell = _rect(x, origin_y, DAY_WIDTH, DAY_HEIGHT,
                     fill=GRID_COLOUR, outline=TEXT_COLOUR)
        all_shapes.append(cell)
        label = _text(x + 1.0, origin_y - DAY_HEIGHT * 0.6, dw,
                      font=FONT_NAME, size=FONT_SIZE_DAY - 1)
        all_shapes.append(label)

    # Day cells
    cal = calendar.monthrange(year, month)
    first_weekday, num_days = cal  # 0=Mon .. 6=Sun
    start_y = origin_y - DAY_HEIGHT

    for day in range(1, num_days + 1):
        idx = first_weekday + day - 1
        col = idx % 7
        row = idx // 7
        x = origin_x + col * DAY_WIDTH
        y = origin_y - (row + 1) * DAY_HEIGHT

        is_weekend = (idx % 7) >= 5
        fill_c = WEEKEND_COLOUR if is_weekend else DAY_COLOUR
        cell = _rect(x, y, DAY_WIDTH, DAY_HEIGHT,
                     fill=fill_c, outline=GRID_COLOUR)
        all_shapes.append(cell)

        day_label = _text(x + 1.0, y + 2.0, str(day),
                          font=FONT_NAME, size=FONT_SIZE_DAY, bold=True)
        all_shapes.append(day_label)

    return all_shapes


# ---------------------------------------------------------------------------
# Full year calendar
# ---------------------------------------------------------------------------

def build_year_grid(year, origin_x, origin_y):
    """Build a 12-month year calendar in a 3×4 grid."""
    cols = 3
    rows = 4
    all_shapes = []

    # Year title
    year_title = _text(origin_x, origin_y + DAY_HEIGHT * 2,
                       str(year), font=FONT_NAME, size=FONT_SIZE_YEAR, bold=True)
    all_shapes.append(year_title)
    grafeno.set_fill(year_title.id, HEADER_COLOUR[0], HEADER_COLOUR[1], HEADER_COLOUR[2])

    month_w = DAY_WIDTH * 7 + MONTH_GAP_X
    month_h = DAY_HEIGHT * 6 + MONTH_GAP_Y

    for month in range(1, 13):
        col = (month - 1) % cols
        row = (month - 1) // cols
        ox = origin_x + col * (month_w + MONTH_GAP_X)
        oy = origin_y - row * (month_h + MONTH_GAP_Y) - DAY_HEIGHT * 2
        all_shapes.extend(build_month_grid(year, month, ox, oy))

    return all_shapes


# ---------------------------------------------------------------------------
# Public API
# ---------------------------------------------------------------------------

def generate_calendar(year=None, month=None, layout="month"):
    """
    Generate a calendar in the active drawing.

    Parameters
    ----------
    year : int
        Calendar year (default: current year).
    month : int or None
        If given, only that month is generated.
        If None and layout=="year", all 12 months are generated.
    layout : str
        "month"  – single month (requires month)
        "year"   – full-year grid
    """
    import datetime as _dt
    if year is None:
        year = _dt.date.today().year

    grafeno.comment(f"Generating calendar for {year}")

    if layout == "year":
        shapes = build_year_grid(year, MARGIN_LEFT, MARGIN_TOP + 30)
    else:
        if month is None:
            month = _dt.date.today().month
        shapes = build_month_grid(year, month, MARGIN_LEFT, MARGIN_TOP)

    # Group everything
    if shapes:
        _group([s.id for s in shapes if hasattr(s, 'id')])
        grafeno.comment(f"Created calendar with {len(shapes)} shapes")
    else:
        grafeno.comment("No shapes created (check Grafeno connection)")

    return shapes


# ---------------------------------------------------------------------------
# Entry point for Grafeno Vetor
# ---------------------------------------------------------------------------

def run():
    """Run the calendar generator with default params (current year, full layout)."""
    generate_calendar(layout="year")
