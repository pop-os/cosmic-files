#!/usr/bin/python3
"""Small, event-driven layer-shell button; no pointer polling or animations."""
import logging
import subprocess
import threading
from pathlib import Path
import gi
gi.require_version("Gdk", "3.0")
gi.require_version("Gtk", "3.0")
gi.require_version("GtkLayerShell", "0.1")
from gi.repository import Gdk, GLib, Gtk, GtkLayerShell

CONTROL = str(Path.home() / ".local/bin/cosmic-show-desktop-control")

class Corner:
    def __init__(self, monitor):
        self.busy = False
        self.window = Gtk.Window(type=Gtk.WindowType.TOPLEVEL)
        self.window.set_app_paintable(True)
        visual = self.window.get_screen().get_rgba_visual()
        if visual:
            self.window.set_visual(visual)
        GtkLayerShell.init_for_window(self.window)
        GtkLayerShell.set_namespace(self.window, "th3al3x-show-desktop")
        GtkLayerShell.set_monitor(self.window, monitor)
        GtkLayerShell.set_layer(self.window, GtkLayerShell.Layer.OVERLAY)
        GtkLayerShell.set_keyboard_mode(self.window, GtkLayerShell.KeyboardMode.NONE)
        GtkLayerShell.set_exclusive_zone(self.window, 0)
        for edge in (GtkLayerShell.Edge.BOTTOM, GtkLayerShell.Edge.RIGHT):
            GtkLayerShell.set_anchor(self.window, edge, True)
        button = Gtk.Button()
        button.set_name("desktop-corner")
        button.set_size_request(10, 36)
        button.set_can_focus(False)
        button.set_tooltip_text("Mostrar escritorio / Restaurar ventanas")
        button.connect("clicked", self.toggle)
        self.window.add(button)
        self.window.show_all()

    def toggle(self, _button):
        if self.busy:
            return
        self.busy = True
        threading.Thread(target=self.run_toggle, daemon=True).start()

    def run_toggle(self):
        try:
            result = subprocess.run([CONTROL, "--toggle"], capture_output=True,
                                    text=True, timeout=5)
            if result.returncode:
                logging.error("Show desktop: %s", result.stderr.strip())
        except (OSError, subprocess.TimeoutExpired):
            logging.exception("Show desktop failed")
        finally:
            GLib.idle_add(self.clear_busy)

    def clear_busy(self):
        self.busy = False
        return False

css = Gtk.CssProvider()
css.load_from_data(b"""
window { background-color: transparent; }
#desktop-corner { min-width: 0; min-height: 0; padding: 0; margin: 0;
  border-radius: 0; border: 0; border-left: 1px solid rgba(255,255,255,0.35);
  background-image: none; background-color: rgba(35,25,50,0.18); box-shadow: none; }
#desktop-corner:hover { background-color: rgba(180,145,240,0.45); }
""")
Gtk.StyleContext.add_provider_for_screen(Gdk.Screen.get_default(), css,
                                       Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION)
display = Gdk.Display.get_default()
corners = []
def rebuild(*_args):
    for corner in corners:
        corner.window.destroy()
    corners[:] = [Corner(display.get_monitor(i)) for i in range(display.get_n_monitors())]
display.connect("monitor-added", rebuild)
display.connect("monitor-removed", rebuild)
rebuild()
Gtk.main()
