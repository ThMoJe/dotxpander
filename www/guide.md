# .XPANDER — User Guide & FAQ
> **Squarespace Page Blueprint**: Use this document to populate the single-page User Guide & FAQ at `https://aivolution.dk/dotxpander`.  
> Each section is marked with its corresponding Squarespace block type for quick copy-pasting.

---

## 🚀 Hero Section

<!-- Squarespace Block: Heading 1 + Subhead (Centered) -->
# Instant Typing Superpowers for Windows.
### Type less, write faster, and automate repetitive text — without scripts, coding, or complexity.

<!-- Squarespace Block: Text (Intro paragraph) -->
**.XPANDER** (dotXPANDER) is a free, ultra-lightweight Windows desktop utility that automatically expands short abbreviations into full sentences, email templates, code blocks, or standard phrases in any application. 

Unlike heavy, complicated tools or scripting engines that require programming knowledge, .XPANDER runs silently in the background, uses virtually **zero memory (~5.8 MB)**, draws **0% CPU** while idle, and requires **no setup** to start saving you time.

---

## 💡 Why .XPANDER?

<!-- Squarespace Block: 3-Column Feature Cards or Grid -->

### 1. Zero Learning Curve
No coding, no macros, and no complicated scripting syntax. If you know how to type an email or fill out a form, you already know how to use .XPANDER. Just pick an abbreviation (like `.em`), assign your full text, and start typing.

### 2. Featherlight & Lightning-Fast
Most modern desktop apps consume hundreds of megabytes of RAM and drain your battery. .XPANDER is engineered in 100% native Rust. It uses **under 6 MB of RAM**—less than a single browser tab—and wakes up in under 50 milliseconds.

### 3. 100% Private & Offline
Your keystrokes and data are your business. .XPANDER runs entirely offline on your computer. It contains **zero telemetry**, sends **no analytics**, and makes **no internet connections**. Your snippets stay securely on your machine.

### 4. Works Everywhere
Whether you are writing an email in Outlook, typing in Word, filling web forms in Chrome or Edge, chatting in Teams or Slack, or coding in Windows Terminal, .XPANDER works seamlessly across all Windows 10 and Windows 11 apps.

---

## 📥 Download & Install

<!-- Squarespace Block: Button Row / Call-to-Action Group -->

Choose the installation method that best fits your workflow:

| Method | Recommended For | Action |
| :--- | :--- | :--- |
| **Microsoft Store** | One-click install with automatic Windows updates. | [Get from Microsoft Store](https://apps.microsoft.com/detail/placeholder-dotxpander) |
| **Windows Setup (.exe)** | Standard installer with optional Start Menu & Desktop shortcuts. | [Download Setup (.exe)](https://github.com/ThMoJe/dotxpander/releases/latest) |
| **Portable Version (.zip)** | Run from a USB drive or work PC without administrator rights. | [Download Portable (.zip)](https://github.com/ThMoJe/dotxpander/releases/latest) |
| **Windows Package Manager** | Command-line installation for power users and IT deployment. | `winget install dotXPANDER` |

> [!NOTE]
> **No Administrator Privileges Required**: Both the Setup Installer and Portable versions install directly to your user profile (`%LOCALAPPDATA%`), meaning you don't need corporate admin rights or IT approval to install and use .XPANDER.

---

## 📖 User Guide: 4 Core Features

<!-- Squarespace Block: Section 1 — Immediate Snippet Expansion -->
### 1. Instant Text Expansion (Type & Expand)

Turn short keystroke triggers into full text instantly anywhere you type.

<!-- Visual Storyboard Callout:
[GIF Placeholder: assets/guide-immediate-expansion.gif]
- Recording Length: ~4 seconds looping.
- Visual: User opens Notepad, types ".em", and it instantly morphs into "john.doe@company.com". Then user types ".sig", and it expands into a formatted 4-line email signature.
-->

#### How to use it:
1. Open the .XPANDER Settings window (press `Win + Alt + X` or click the `.X` icon in your Windows system tray).
2. Click **Add Snippet** (+).
3. Enter your **Trigger** (for example: `.em`) and your **Replacement Text** (for example: `john.doe@example.com`).
4. Keep the mode as **Immediate**.
5. Switch to any app (Notepad, Outlook, Word, Browser) and type `.em` — it instantly expands!

> **💡 Pro-Tip**: Starting your triggers with a punctuation symbol like a dot (`.`), comma (`,`), or slash (`/`) ensures that you don't accidentally expand words while typing normal sentences (e.g. use `.sig` instead of `sig`).

---

<!-- Squarespace Block: Section 2 — Hotkey Expansion -->
### 2. On-Demand Hotkey Expansion

Prefer to trigger replacements manually rather than automatically? Hotkey mode lets you type a keyword and press a keyboard shortcut to expand it only when you want.

<!-- Visual Storyboard Callout:
[GIF Placeholder: assets/guide-hotkey-expansion.gif]
- Recording Length: ~4 seconds looping.
- Visual: User types "meeting_notes" into an email, presses Ctrl+Shift+T, and the text transforms into a bulleted template.
-->

#### How to use it:
1. In Settings, create or edit a snippet and set its mode to **Hotkey**.
2. Type the keyword anywhere.
3. Press **`Ctrl + Shift + T`** (or your custom shortcut configured in Settings) to expand it on demand.

---

<!-- Squarespace Block: Section 3 — Case & Space Changer -->
### 3. Case & Space Changer (`Ctrl + CapsLock`)

Fix accidental CAPS LOCK typing, reformat headings, or clean up file names instantly without retyping a single word.

<!-- Visual Storyboard Callout:
[GIF Placeholder: assets/guide-case-changer.gif]
- Recording Length: ~5 seconds looping.
- Visual: User highlights "PROJECT PROPOSAL DRAFT 2026", presses Ctrl+CapsLock, a sleek dark menu appears, clicks "Title Case", and the highlighted text transforms to "Project Proposal Draft 2026".
-->

#### How to use it:
1. **Highlight any text** in any program.
2. Press **`Ctrl + CapsLock`**.
3. A popup menu appears with one-click conversion options:
   - **UPPERCASE**, **lowercase**, **Title Case**, **Sentence case**
   - **camelCase**, **PascalCase**, **snake_case**, **kebab-case**
   - **Clean Filename**: Strips invalid Windows characters (`\ / : * ? " < > |`) so text can be safely used as a file or folder name.
4. Click your desired format, and your text updates instantly!

---

<!-- Squarespace Block: Section 4 — Quick Switch File Dialog Navigation -->
### 4. Quick Switch (Open/Save File Dialog Navigation)

Never waste time clicking through deep folder trees in Windows Open or Save dialogs again.

<!-- Visual Storyboard Callout:
[GIF Placeholder: assets/guide-quick-switch.gif]
- Recording Length: ~5 seconds looping.
- Visual: A browser shows an "Upload File" dialog stuck in "Downloads". User Alt+Tabs to File Explorer where their "Work/2026/Reports" folder is open, then Alt+Tabs back to the browser: the file dialog automatically jumps to "Work/2026/Reports".
-->

#### How to use it:
1. When any standard Windows **Open** or **Save As** dialog is on your screen:
2. Have the folder you want already open in **File Explorer**.
3. Simply **Alt + Tab** to File Explorer and back to your dialog (or click between them).
4. .XPANDER detects your active Explorer folder and automatically navigates the Open/Save dialog directly to that folder.

---

## ❓ Frequently Asked Questions (FAQ)

<!-- Squarespace Block: Accordion Block -->

#### Why didn't my snippet expand when I typed it?
Here are the three most common reasons:
1. **Typing too slowly**: .XPANDER tracks keystrokes within a natural typing window. If you pause for more than a few seconds in the middle of typing your trigger, retype it cleanly.
2. **Administrator Permissions**: If you are typing inside an application running with Administrator rights (such as an elevated Command Prompt or regedit), Windows security blocks standard utilities from sending keystrokes. You can right-click .XPANDER and choose *"Run as administrator"* if you need it in elevated apps.
3. **Trigger Mode**: Check whether your snippet is set to *Immediate* (expands automatically) or *Hotkey* (requires pressing `Ctrl + Shift + T`).

#### Does .XPANDER record or store my keystrokes?
**No. Absolutely not.** .XPANDER operates with zero network connectivity and zero telemetry. It keeps only the last few characters you typed in a tiny, temporary memory ring in your computer's RAM solely to detect your trigger phrase. As soon as you type the next letter, the previous characters are discarded. Nothing is ever written to a log file or transmitted over the internet.

#### Can I sync my snippets across my work and home computers?
**Yes!** .XPANDER stores all your snippets in a simple, portable file called `config.toml`.
1. Open .XPANDER Settings and go to the **General** tab.
2. Click **Move Config File…**.
3. Choose a folder inside your **OneDrive**, **Dropbox**, **Google Drive**, or **iCloud Drive**.
4. Repeat this on your other computers and select the same folder. Your snippets will automatically stay in sync across all your devices!

#### Can I use .XPANDER on a work PC without installing it?
**Yes.** Download the **Portable ZIP** release. Extract the folder onto your desktop, Documents folder, or a USB flash drive, and double-click `dotxpander.exe`. It runs self-contained without writing anything to the Windows Registry or requiring administrator credentials.

#### How do I temporarily pause .XPANDER if I need to type triggers normally?
You can temporarily pause snippet expansion at any time:
- Right-click the `.X` icon in your Windows system tray and select **Pause Expansion**.
- To resume, click it again to uncheck **Pause**.

#### Where can I get help or report an issue?
If you run into an issue, have a feature suggestion, or have a question that isn't answered here:
- **Email Support**: [info@aivolution.dk](mailto:info@aivolution.dk)
- **Community & Bug Reports**: Open an issue on our [GitHub Issue Tracker](https://github.com/ThMoJe/dotxpander/issues).

---

## 🔗 Project Links & Open Source

<!-- Squarespace Block: Links / Footer -->

.XPANDER is built with passion and distributed under the permissive **MIT License**.

- **Source Code**: [GitHub Repository](https://github.com/ThMoJe/dotxpander)
- **Release Downloads**: [Latest Releases](https://github.com/ThMoJe/dotxpander/releases)
- **License**: [MIT License](https://github.com/ThMoJe/dotxpander/blob/main/LICENSE)
- **Publisher**: [aiVOLUTION](https://aivolution.dk)
