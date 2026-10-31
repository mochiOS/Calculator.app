# Calculator

A native AppCore and ViewKit calculator. Standard, Scientific, Convert and
Programmer modes share the existing exact-arithmetic engine and saved history.

The toolbar uses ViewKit's ActionToolbar with native mode and overflow menus.
Mode settings use standard pickers, segmented controls and checkboxes. The
keypad retains its calculator-specific arrangement and double-width zero key,
using standard buttons, theme typography, spacing and corners. Dense keypads
use the shared no-shadow option to stay within the renderer's geometry budget.
History has a separate window with a standard Scroll and ListRow collection.

Typing returns focus to the expression after toolbar interactions. Enter
calculates; Escape clears; Ctrl+1 through Ctrl+4 select modes; Ctrl+H opens
history; Ctrl+Z and Ctrl+Shift+Z undo and redo; Ctrl+Shift+C copies the result.
The native action menu also offers expression copying and explicit selection
of the startup mode. Changing modes preserves each mode's current expression.

The default window remains 360 × 540 logical pixels. Small toolbars overflow
secondary actions instead of overlapping the mode selector. Start-mode settings
and history in Linux previews are isolated by MOCHIOS_PREVIEW_CONFIG_ROOT.
