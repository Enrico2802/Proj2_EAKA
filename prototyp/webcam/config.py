"""Central configuration of the webcam control.

Constants only - no logic. All other modules import from here.
The thresholds are RATIOS (share of active cells per zone) and therefore
independent of the grid size (see the concept document, section 4.1).
"""

# Camera / image
CAMERA_INDEX        = 0      # OpenCV index (not the USB VID/PID!). 0 = default camera.
# Full HD runs at ~30 FPS (pipeline incl. MOG2, measured). This requires the
# MJPG format, which webcam_source.py enforces - otherwise the cam falls back
# to YUY2 (~5 FPS). Alternatives: 1280x720 (cheap), 2560x1440 (~27 FPS).
FRAME_WIDTH         = 1920
FRAME_HEIGHT        = 1080
MIRROR              = True   # selfie view: left in the image = the person's left

# Grid
GRID_COLS           = 32
GRID_ROWS           = 24
CELL_ACTIVE_THRESH  = 40     # mask value 0..255 above which a cell counts as "active"

# Detector thresholds (ratios 0..1)
ZONE_ENTER_RATIO    = 0.15   # ratio of active cells -> zone "on"
ZONE_EXIT_RATIO     = 0.08   # hysteresis -> zone "off"
COOLDOWN_S          = 0.5    # lockout per tap gesture against double triggering
CALIBRATION_S       = 1.0    # duration of the startup calibration
FPS_ESTIMATE        = 30     # used only to derive calib_frames from CALIBRATION_S

# MOG2 background subtraction
MOG2_HISTORY        = 300
MOG2_VAR_THRESHOLD  = 25
MOG2_DETECT_SHADOWS = True

# KeySender
TAP_HOLD_MS         = 40     # hold duration of a tap in milliseconds

# Overlay
MONITOR2_X_OFFSET   = 1920   # x position of the overlay window (2nd monitor)

# Gesture -> key
KEYS = {"left": "a", "right": "d", "up": "space", "down": "s"}

# Zones as relative rectangles (0..1), format (x0, y0, x1, y1).
# Do NOT let them overlap! The center area stays a neutral idle zone.
ZONES = {
    "up":    (0.34, 0.00, 0.66, 0.38),   # top center  -> jump (space)
    "left":  (0.00, 0.00, 0.34, 0.70),   # left        -> A
    "right": (0.66, 0.00, 1.00, 0.70),   # right       -> D
    "down":  (0.00, 0.70, 1.00, 1.00),   # bottom band -> S (crouch, HOLD)
}

# Derived values
CALIB_FRAMES = round(CALIBRATION_S * FPS_ESTIMATE)
