---
name: CCR UI
description: Operational console for advanced AI CLI configuration, runtime monitoring, sync, and usage insight workflows.
colors:
  clay-accent: "#d97757"
  clay-accent-hover: "#e48667"
  clay-accent-active: "#c96c4d"
  sand-accent: "#b99666"
  success: "#5b8a62"
  warning: "#bc8540"
  danger: "#c76953"
  info: "#7d97b6"
  neutral-base: "#f4ede3"
  neutral-elevated: "#fbf6ee"
  neutral-surface: "#fffaf3"
  neutral-overlay: "#ede2d4"
  ink-primary: "#31241c"
  ink-secondary: "#5f4d3f"
  ink-muted: "#7f6a5b"
  dark-base: "#17120f"
  dark-surface: "#2a221e"
  dark-ink-primary: "#f3eadf"
typography:
  headline:
    fontFamily: "MapleBright, SF Pro Display, SF Pro Text, PingFang SC, Microsoft YaHei UI, Microsoft YaHei, sans-serif"
    fontSize: "1.5rem"
    fontWeight: 600
    lineHeight: 1.2
    letterSpacing: "-0.01em"
  title:
    fontFamily: "MapleBright, SF Pro Text, PingFang SC, Microsoft YaHei UI, Microsoft YaHei, sans-serif"
    fontSize: "1.0625rem"
    fontWeight: 600
    lineHeight: 1.3
    letterSpacing: "0"
  body:
    fontFamily: "MapleBright, SF Pro Text, PingFang SC, Microsoft YaHei UI, Microsoft YaHei, sans-serif"
    fontSize: "1rem"
    fontWeight: 400
    lineHeight: 1.56
    letterSpacing: "0"
  label:
    fontFamily: "MapleBright, SF Pro Text, PingFang SC, Microsoft YaHei UI, Microsoft YaHei, sans-serif"
    fontSize: "0.8125rem"
    fontWeight: 500
    lineHeight: 1.24
    letterSpacing: "0"
rounded:
  sm: "6px"
  md: "8px"
  lg: "12px"
  xl: "16px"
  pill: "9999px"
spacing:
  xs: "0.25rem"
  sm: "0.5rem"
  md: "1rem"
  lg: "1.5rem"
  xl: "2rem"
  xxl: "4rem"
components:
  button-primary:
    backgroundColor: "{colors.clay-accent}"
    textColor: "{colors.neutral-surface}"
    rounded: "{rounded.pill}"
    padding: "0.625rem 1rem"
    height: "44px"
  button-secondary:
    backgroundColor: "{colors.neutral-elevated}"
    textColor: "{colors.ink-primary}"
    rounded: "{rounded.md}"
    padding: "0.625rem 1rem"
    height: "44px"
  input-default:
    backgroundColor: "{colors.neutral-elevated}"
    textColor: "{colors.ink-primary}"
    rounded: "{rounded.md}"
    padding: "0.625rem 1rem"
    height: "44px"
  card-elevated:
    backgroundColor: "{colors.neutral-surface}"
    textColor: "{colors.ink-primary}"
    rounded: "{rounded.xl}"
    padding: "1.5rem"
  nav-item-active:
    backgroundColor: "{colors.neutral-surface}"
    textColor: "{colors.ink-primary}"
    rounded: "{rounded.lg}"
    padding: "0.5rem 0.75rem"
---

# Design System: CCR UI

## 1. Overview

**Creative North Star: "The Editorial Control Room"**

CCR UI is a dense operational workbench with an editorial surface system: quiet shell, precise labels, warm clay accent, and structured panels that help expert users see state quickly. The interface should feel trusted during configuration, sync, command execution, and usage analysis, where unclear state can cost real local data.

The visual language is restrained by default. Color marks action, current selection, platform identity, and semantic state. Surface treatment may use the existing structural glass tokens for shell layering, but heavy decorative glass, anime atmosphere, catgirl or NEKO branding, guofeng branches, and purple-tech gradients are legacy directions to remove over time.

**Key Characteristics:**

- Dense but organized workbench layouts.
- Warm clay accent used sparingly for primary action and active state.
- Charcoal or high-contrast ink over neutral surfaces.
- Consistent rounded product controls, usually 6px to 16px, with pill buttons where actions need strong hit targets.
- Motion limited to state change, feedback, route continuity, and loading.

## 2. Colors

The palette is a restrained editorial system: neutral shell layers carry most of the UI, clay carries action, and semantic colors carry operational state.

### Primary

- **Clay Accent**: The primary action and active-selection color. Use it for primary buttons, focus rings, active navigation, and critical emphasis. It must stay rare enough to remain meaningful.

### Secondary

- **Warm Sand**: Secondary accent for supporting emphasis, quiet charts, and paired accent moments. Do not use it as a competing brand axis.
- **Platform Colors**: Claude clay, Codex sage, and Gemini or Antigravity blue exist for recognition only. They should identify tool context, not repaint whole screens.

### Tertiary

- **Operational Semantics**: Success, warning, danger, and info colors are for system state, validation, health, and usage signals. They should be visible but not saturated across inactive surfaces.

### Neutral

- **Editorial Base**: The warm clay theme uses neutral base, elevated, surface, and overlay layers to separate app shell, workspace, cards, and overlays.
- **Light Runtime Neutral**: The active light theme can resolve to a cooler near-white base. Preserve this as a product reading surface, not a blank SaaS canvas.
- **Dark Runtime Neutral**: Dark mode uses near-black clay-brown surfaces and warm ink. Contrast must remain high for labels, data, and placeholders.

### Named Rules

**The Accent Scarcity Rule.** Clay accent should occupy less than 10% of a normal screen. If it appears everywhere, nothing reads as active.

**The Legacy Branch Rule.** Do not add new purple-tech, guofeng, anime, catgirl, or NEKO color variants. Existing traces are migration debt, not optional themes.

## 3. Typography

**Display Font:** MapleBright with SF Pro Display and system CJK fallbacks.
**Body Font:** MapleBright with SF Pro Text and system CJK fallbacks.
**Label/Mono Font:** The current mono token aliases MapleBright and system UI fallbacks.

**Character:** The system uses one tuned sans stack for product focus. Hierarchy comes from weight, size, spacing, and layout, not from decorative font changes.

### Hierarchy

- **PageTitle** (600, 1.5rem, 1.2, tracking -0.01em): Page titles. Use a fixed rem size.
- **SectionTitle** (600, 1.0625rem, 1.3): Section titles, card titles, dialog headings, and dense dashboard blocks.
- **Body** (400, 1rem, 1.56, tracking 0): Prose, descriptions, hints, and operational explanations. Cap long prose at 65 to 75ch.
- **Label** (500, 0.8125rem, 1.24, tracking 0): Form labels, small controls, metadata, and table labels. Uppercase is allowed only for short Latin system labels.

### Named Rules

**The Product Type Rule.** Do not use fluid hero typography inside app surfaces. Fixed rem scales serve this tool better than viewport-driven display type.

**The Label Clarity Rule.** Labels must stay readable before they look elegant. Muted labels still need AA contrast against their surface.

## 4. Elevation

CCR UI uses a hybrid of tonal layering, subtle borders, and a small shadow vocabulary. Surfaces are mostly defined by background role and border strength. Shadows are for shell separation, hover response, modal depth, and card hierarchy, not decoration.

### Shadow Vocabulary

- **Subtle Rest** (`--shadow-sm`): Optional hover or field separation. Resting cards use no shadow, or only this step.
- **Structural Overlay** (`--shadow-md` / `--elevation-2`): Sticky bars and interactive overlays. Do not use this on resting cards.
- **Modal Depth** (`--shadow-lg` and modal surface tokens): Overlays and dialogs that must separate from the workspace.
- **Semantic Glow** (`--glow-primary`, `--glow-success`, `--glow-warning`, `--glow-danger`, `--glow-info`): Focus rings only. Do not use decorative glow.

### Named Rules

**The Structural Glass Rule.** Existing glass tokens may define shell, workspace, card, modal, and status layers. Do not add decorative glass cards for visual novelty.

**The No Ghost Card Rule.** Avoid pairing a 1px border with a wide decorative drop shadow on ordinary repeated cards. Use tonal layering first.

## 5. Components

### Buttons

- **Shape:** Pill buttons for primary product actions, with a minimum 44px height. Secondary buttons use 8px to 10px corners.
- **Primary:** Solid `--color-accent-primary` fill with inverted text. Do not use a gradient. Use only for the next meaningful command.
- **Hover / Focus:** Hover lifts by less than 1px. Focus uses a visible clay ring. Loading disables click behavior and sets `aria-busy`.
- **Secondary / Ghost / Outline:** Secondary uses neutral elevated surfaces and 8px to 10px corners. Ghost is quiet text-first action. Outline is for low-emphasis actions and filters.

### Chips

- **Style:** Small neutral or semantic pills with clear text contrast and minimal border.
- **State:** Selected chips may use clay tint or semantic tint, but inactive chips stay neutral.

### Cards / Containers

- **Corner Style:** Standard cards use 12px to 16px. Avoid 20px plus card radii unless the component is already a major shell panel.
- **Background:** Use workspace, card, modal, and status surface tokens. Do not nest card-looking surfaces inside other card-looking surfaces.
- **Shadow Strategy:** Resting cards use no shadow, or only `--shadow-sm`. Interactive cards may lift by 1px to 2px with a subtle border shift.
- **Internal Padding:** Dense cards usually use 1rem to 1.5rem. Large dashboard panels may use 2rem.

### Inputs / Fields

- **Style:** Rounded 8px to 10px fields with neutral surface backgrounds, clear borders, and 44px minimum height.
- **Focus:** Clay border and visible focus ring. Do not use decorative glow.
- **Error / Disabled:** Error uses danger color plus `aria-invalid`. Disabled controls reduce opacity while preserving label readability.

### Navigation

Navigation is dense, persistent, and grouped by operational area. Active items use clear text contrast, neutral surface, and restrained accent. Mobile navigation uses a modal sidebar with an explicit backdrop and close button.

### Workbench Shell

The shell combines resizable navigation, sticky topbar, scrollable content, route transitions, backend status banners, toast feedback, and confirm dialogs. Shell surfaces must prioritize readability and route continuity over decorative background effects.

## 6. Do's and Don'ts

### Do:

- **Do** keep expert density and explicit operational scope visible.
- **Do** use clay accent for primary action, active navigation, focus, and critical emphasis.
- **Do** preserve light and dark contrast for body text, placeholders, labels, and status text.
- **Do** use structural surface roles: shell, workspace, card, modal, and status.
- **Do** keep motion between 100ms and 300ms for product state changes, with reduced-motion alternatives.
- **Do** treat Tauri-only failures in web preview as runtime limitations when the task is not web compatibility.

### Don't:

- **Don't** add generic SaaS admin styling, purple or purple-blue gradients, or marketing hero patterns.
- **Don't** add heavy Liquid Glass or glassmorphism as decoration.
- **Don't** add guofeng, neko, anime, catgirl, or mascot-heavy variants.
- **Don't** hide operational risk behind beginner-friendly simplification.
- **Don't** create nested cards or identical decorative card grids.
- **Don't** use side-stripe accent borders, gradient text, or 32px plus radii on product cards.
