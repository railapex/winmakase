# Winsome

> An omakase, omarchy-inspired desktop environment for Windows 11 — tiling, coordinated themes, and one keymap your hands can take with you to Linux.

**Status: pre-alpha.** Nothing to install yet. Design and construction happening in the open — see [docs/DESIGN.md](docs/DESIGN.md) and [docs/TODO.md](docs/TODO.md).

## What

[Omarchy](https://omarchy.org) proved that an opinionated, curated desktop — one command, one keymap, one theme switch — beats a pile of dotfiles. [Omacosy](https://github.com/paulsp94/omacosy) brought it to macOS. Winsome brings it to Windows 11:

- **Tiling** via [GlazeWM](https://github.com/glzr-io/glazewm), driven by omarchy's exact keybinding grammar
- **A real modifier key** — Caps Lock (or the context-menu key) becomes the window-manager mod via [kanata](https://github.com/jtroo/kanata) tap-hold: tap for your launcher, hold for the WM. On a future omarchy machine, `caps:super` gives you the identical physical motion — the muscle memory transfers
- **Omarchy theme compatibility** — Winsome consumes real omarchy theme folders and renders them onto Windows Terminal, the status bar, window borders, wallpaper, Neovim, and Windows light/dark mode. Stock and community omarchy themes just work
- **Status bar** via [Zebar](https://github.com/glzr-io/zebar), taskbar hidden
- **Stock Windows where Windows already wins** — PowerToys Run/Command Palette, Win+V clipboard history, built-in OCR and dictation. Winsome curates; it doesn't rebuild
- **A supervisor** that keeps the stack alive, restarts the tiler on crash or monitor replug, and shows health in the bar — silent failure is the enemy
- **Reversible** — manifest-driven uninstall restores your machine; `winsome toggle off` disables everything without uninstalling

## What it deliberately is not

- Not a fork of omarchy — a sibling implementation that treats omarchy's themes and keymap as a compatibility target
- Not a titlebar remover — no Windows tiler strips window chrome (neither does macOS); curated apps get per-app compact-chrome settings instead
- Not a notification theming tool — Windows toasts aren't themeable; that's a documented gap, same as omarchy's unthemed GUI apps

## Credits

Standing on: [omarchy](https://github.com/basecamp/omarchy) (DHH / Basecamp), [omacosy](https://github.com/paulsp94/omacosy) (Paul Springer), [GlazeWM & Zebar](https://github.com/glzr-io) (glzr.io), [kanata](https://github.com/jtroo/kanata) (jtroo).

## License

MIT
