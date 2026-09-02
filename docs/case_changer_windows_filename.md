# Case/Space Changer: Complete Transformations Reference & Manual

> **Document Purpose**: Comprehensive reference guide and user manual specification for all text transformations in dotXPANDER's **Case/Space Changer** menu. Intended as a source for user documentation, in-app help, and FAQs.

---

## 1. Overview & How It Works

The **Case/Space Changer** is a global productivity tool built into dotXPANDER that allows users to instantly reformat, recase, sanitize, or clean up any text selected in any Windows application.

### How to Use
1. **Select** text in any application (browser, text editor, word processor, terminal, Explorer, etc.).
2. **Press `Ctrl+CapsLock`** to trigger the Win32 popup menu at the current mouse cursor location.
3. **Select** the desired transformation with the mouse or by pressing its underlined **accelerator key**.
4. dotXPANDER transforms the text in-place and restores your previous clipboard contents so your clipboard history remains intact.

---

## 2. Menu Structure & Shortcuts

The menu is organized into four distinct visual sections:

```
┌──────────────────────────────────────┐
│  &UPPERCASE                   (U)    │
│  &lowercase                   (L)    │
│  &Title Case                  (T)    │
│  Se&ntence case               (N)    │
├──────────────────────────────────────┤
│  &Fix Linebreaks              (F)    │
│  &Spaces                    > (S)    │
│    ├── &Remove spaces         (R)    │
│    ├── Replace space with _   (_)    │
│    └── Replace space with -   (-)    │
├──────────────────────────────────────┤
│  lower&CamelCase              (C)    │
│  &PascalCase                  (P)    │
├──────────────────────────────────────┤
│  Windows &filename            (F)    │
└──────────────────────────────────────┘
```

### Bilingual Menu Labels & Accelerator Keys

| Menu Item | English Label | EN Key | Danish Label | DA Key |
|:---|:---|:---:|:---|:---:|
| **Uppercase** | `&UPPERCASE` | `U` | `&STORE BOGSTAVER` | `S` |
| **Lowercase** | `&lowercase` | `L` | `s&må bogstaver` | `M` |
| **Title Case** | `&Title Case` | `T` | `S&tort Forbogstav` | `T` |
| **Sentence Case** | `Se&ntence case` | `N` | `Sæt&nings-format` | `N` |
| **Fix Linebreaks** | `&Fix Linebreaks` | `F` | `&Fix Linjeskift` | `F` |
| **Spaces Submenu** | `&Spaces` | `S` | `&Mellemrum` | `M` |
| ↳ *Remove Spaces* | `&Remove spaces` | `R` | `&Fjern mellemrum` | `F` |
| ↳ *Space to Underscore* | `Replace space with _` | `_` | `Erstat mellemrum med _` | `_` |
| ↳ *Space to Dash* | `Replace space with -` | `-` | `Erstat mellemrum med -` | `-` |
| **lowerCamelCase** | `lower&CamelCase` | `C` | `lower&CamelCase` | `C` |
| **PascalCase** | `&PascalCase` | `P` | `&PascalCase` | `P` |
| **Windows Filename** | `Windows &filename` | `F` | `Windows &filnavn` | `F` |

> [!NOTE]
> For **Replace space with _** and **Replace space with -**, dotXPANDER uses custom `WM_MENUCHAR` handling so you can press `_` or `-` directly on your keyboard to trigger them immediately without needing an underlined mnemonic character.

---

## 3. Transformation Details

### 3.1 Case Transformations

#### `UPPERCASE`
- **Description**: Converts all characters in the text to uppercase using full Unicode rules.
- **Rules**: Correctly maps international characters (e.g. `æøå` → `ÆØÅ`, `äöü` → `ÄÖÜ`, `ß` → `SS`).
- **Example**: `hello world ændring` → `HELLO WORLD ÆNDRING`

#### `lowercase`
- **Description**: Converts all characters in the text to lowercase using full Unicode rules.
- **Rules**: Correctly maps international characters (e.g. `ÆØÅ` → `æøå`, `ÄÖÜ` → `äöü`).
- **Example**: `HELLO WORLD ÆNDRING` → `hello world ændring`

#### `Title Case`
- **Description**: Capitalizes the first character of every whitespace-separated word.
- **Rules**: Existing internal capitalization within words is preserved.
- **Example**: `hello world from denmark` → `Hello World From Denmark`

#### `Sentence case`
- **Description**: Capitalizes the first letter of each sentence and lowercases the rest.
- **Rules**: Detects sentence boundaries after `.`, `!`, and `?` followed by whitespace.
- **Example**: `hello world. this is a TEST! another sentence? yes.` → `Hello world. This is a test! Another sentence? Yes.`

#### `lowerCamelCase`
- **Description**: Converts text into standard programming `lowerCamelCase`.
- **Rules**:
  - Splits input on whitespace, underscores (`_`), hyphens (`-`), and existing camelCase/PascalCase boundaries (transitions from lowercase to uppercase).
  - The first word is entirely lowercased.
  - Every subsequent word has its first grapheme capitalized and the rest lowercased.
- **Example**: `hello world_user-name` → `helloWorldUserName`
- **Example**: `PascalCaseInput` → `pascalCaseInput`

#### `PascalCase`
- **Description**: Converts text into standard programming `PascalCase` (UpperCamelCase).
- **Rules**:
  - Uses the same word-splitting logic as `lowerCamelCase`.
  - Every word has its first grapheme capitalized and remaining characters lowercased.
- **Example**: `hello world_user-name` → `HelloWorldUserName`
- **Example**: `lowerCamelInput` → `LowerCamelInput`

---

### 3.2 Spacing & Linebreak Transformations

#### `Fix Linebreaks`
- **Description**: Normalizes all line endings in the selected text to Windows standard CRLF (`\r\n`).
- **Rules**:
  - Eliminates mixed line breaks (`\r\n` and lone `\n`).
  - Converts Unix-style `\n` to Windows standard `\r\n` without double-converting existing `\r\n`.
  - Conforms to the Win32 `CF_UNICODETEXT` clipboard specification.
- **Example**: `line1\nline2\r\nline3` → `line1\r\nline2\r\nline3`

#### `Remove spaces`
- **Description**: Strips all spaces, tabs, and horizontal whitespace from the text.
- **Rules**:
  - Linebreaks (`\r`, `\n`) are preserved so multi-line text structures stay intact.
- **Example**: `hello   world \t 123` → `helloworld123`

#### `Replace space with _` (Underscore)
- **Description**: Replaces all horizontal spaces and tabs with an underscore (`_`).
- **Rules**:
  - Linebreaks are preserved.
  - Consecutive spaces produce consecutive underscores (`a   b` → `a___b`).
- **Example**: `Project Plan 2026` → `Project_Plan_2026`

#### `Replace space with -` (Dash / Hyphen)
- **Description**: Replaces all horizontal spaces and tabs with a hyphen (`-`).
- **Rules**:
  - Linebreaks are preserved.
  - Consecutive spaces produce consecutive hyphens (`a   b` → `a---b`).
- **Example**: `Project Plan 2026` → `Project-Plan-2026`

---

### 3.3 Filename Sanitization

#### `Windows filename`
- **Description**: Sanitizes selected text into a safe, valid Windows file or folder name.
- **Summary of Decisions**:
  - **Forbidden characters stripped**: `< > : " / \ | ? *` and ASCII control characters (0–31) are completely removed without adding dashes or underscores.
  - **International characters preserved**: Native support for Danish `æ`, `ø`, `å`, accented letters, Asian characters, etc.
  - **Whitespace collapsed**: Normal spaces are kept; consecutive spaces, tabs, and newlines (`\r\n`, `\n`) are collapsed into a single space.
  - **Edge trimming**: Leading and trailing spaces and dots (`.`) are trimmed. Internal dots (e.g. extensions like `.xlsx`) are preserved.
  - **Casing**: Original letter casing is preserved as-is.
  - **Reserved device names**: Left unchanged (not altered).
  - **Length limit**: Truncated to a maximum of 255 characters (Windows NTFS component limit) on safe character boundaries, with trailing spaces/dots re-trimmed.
  - **All-invalid fallback**: If the selection consists only of invalid characters and becomes empty, it prefixes the text with a localized warning:
    - English: `All chars are invalid: <text>`
    - Danish: `Alle tegn er ugyldige: <text>`
- **Example**: `Report: 2026/Q1 *Final*?` → `Report 2026Q1 Final`
- **Example**: `  ...Danish: Ændringer & Økonomi...  ` → `Danish Ændringer & Økonomi`

---

## 4. Comprehensive Transformation Matrix

| Transformation | Input Example | Output Result |
|:---|:---|:---|
| **UPPERCASE** | `hello world æøå` | `HELLO WORLD ÆØÅ` |
| **lowercase** | `HELLO WORLD ÆØÅ` | `hello world æøå` |
| **Title Case** | `the quick brown fox` | `The Quick Brown Fox` |
| **Sentence case** | `HELLO WORLD! this is A test.` | `Hello world! This is a test.` |
| **lowerCamelCase** | `user_first-name` | `userFirstName` |
| **PascalCase** | `user_first-name` | `UserFirstName` |
| **Fix Linebreaks** | `line1\nline2` | `line1\r\nline2` |
| **Remove spaces** | `hello   world` | `helloworld` |
| **Replace space with _** | `hello world 2026` | `hello_world_2026` |
| **Replace space with -** | `hello world 2026` | `hello-world-2026` |
| **Windows filename** | `Plan: 2026/Q1 *Final*?` | `Plan 2026Q1 Final` |
| **Windows filename (Danish)** | `  ...Budget 2026 - Årsrapport...  ` | `Budget 2026 - Årsrapport` |
| **Windows filename (All invalid)** | `:::*?<>` | `All chars are invalid: :::*?<>` |

---

## 5. Frequently Asked Questions (FAQ)

### Q1: What happens to my clipboard when I use the Case Changer?
**A:** dotXPANDER automatically takes a full backup of all clipboard formats (text, rich text, images, custom formats) before copying the selected text. Once the transformed text is pasted into your target application, dotXPANDER restores your original clipboard contents exactly as they were.

### Q2: Why does the popup menu close if I click away or press Escape?
**A:** The menu is designed as a non-intrusive modal popup. If you decide not to transform your selection, pressing `Escape` or clicking anywhere outside the menu dismisses it immediately and restores your clipboard without changing your document.

### Q3: Can I customize or disable the Case Changer?
**A:** Yes. In the dotXPANDER Settings window under the **General** tab, you can toggle the **Case/Space Changer** feature on or off.

### Q4: Does Title Case alter letters inside words?
**A:** No. `Title Case` only capitalizes the first letter of each word; any existing internal capitalization (such as `iPhone` or `dotXPANDER`) is preserved.

### Q5: How does `Windows filename` handle extensions like `.docx` or `.tar.gz`?
**A:** `Windows filename` only strips leading and trailing dots (which Windows prohibits). All internal dots are preserved, so extensions like `document.docx` or `archive.tar.gz` remain intact.
