# WorkWatch

WorkWatch is a terminal-based time tracking tool written in Rust, featuring a TUI (Text-based User Interface) powered by `ratatui`, with support for activity logging, real-time elapsed time tracking, and webhook-based notifications.

---

## ✨ Features

* Clock in / Clock out with Discord webhook integration
* Real-time timer display
* Log management (add/edit/delete)
* Toggle between Menu, Working mode, and Log view
* Keyboard-friendly controls (Vim-like navigation in Logs)
* Environment-based configuration with `.env`

---

## ⚙ Configuration

Create a `.env` file at the root of your project:

```env
WORKWATCH_USERNAME=YourName
WORKWATCH_WEBHOOK=https://discord.com/api/webhooks/... (optional)
```

If `WORKWATCH_WEBHOOK` is not provided, webhook notifications will be disabled.

---

## ⌨ Controls

### Menu

* `C` - Clock In
* `Q` - Quit

### Working

* `L` - View Logs
* `A` - Add Log
* `C` - Clock Out

### Logs

* `T` - Return to Working Mode
* `A` - Add Log
* `E` - Edit Selected Log
* `D` - Delete Selected Log
* `C` - Clock Out
* `Up/K` / `Down/J` - Navigate Logs

---

## ✉ Webhook Messages

When you clock in or out, a rich embed will be sent to your specified webhook URL with:

* Username
* Date and Time
* Elapsed Time (on clock out)
* Activity Logs (on clock out)

---

## ❓ TODO / Improvements

* Statistics / summaries
* Export to CSV
* Unit tests

---
