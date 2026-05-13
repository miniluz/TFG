from matplotlib import rcParams

OUTPUT_DIR = "figures"
DPI = 600

FONT_FAMILY = "sans-serif"
FONT_SIZE = 11
LABEL_SIZE = 12
TITLE_SIZE = 11

LINE_COLOR = "#4E79A7"
AXES_COLOR = "#34495E"
GRID_COLOR = "#9B9B9B"
GRID_STYLE = ":"
FACE_COLOR = "#FFFFFF"

LINE_WIDTH = 2.0
OUTER_LINE_WIDTH = 1.5
GRID_LINE_WIDTH = 0.8
TICK_LENGTH = 5
TICK_WIDTH = 1.0

PADDING = 0.05


def apply_style():
    rcParams["font.family"] = FONT_FAMILY
    rcParams["font.size"] = FONT_SIZE
    rcParams["axes.labelsize"] = LABEL_SIZE
    rcParams["axes.titlesize"] = TITLE_SIZE
    rcParams["axes.edgecolor"] = AXES_COLOR
    rcParams["axes.linewidth"] = OUTER_LINE_WIDTH
    rcParams["axes.facecolor"] = FACE_COLOR
    rcParams["axes.grid"] = True
    rcParams["grid.color"] = GRID_COLOR
    rcParams["grid.linestyle"] = GRID_STYLE
    rcParams["grid.linewidth"] = GRID_LINE_WIDTH
    rcParams["xtick.color"] = AXES_COLOR
    rcParams["ytick.color"] = AXES_COLOR
    rcParams["xtick.major.size"] = TICK_LENGTH
    rcParams["ytick.major.size"] = TICK_LENGTH
    rcParams["xtick.major.width"] = TICK_WIDTH
    rcParams["ytick.major.width"] = TICK_WIDTH
    rcParams["figure.facecolor"] = "white"
