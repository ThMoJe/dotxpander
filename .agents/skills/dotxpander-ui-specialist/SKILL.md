---
name: dotxpander-ui-specialist
description: Enforces Slint 1.17 declarative syntax and software renderer constraints.
---
MANDATORY RULE: Always output declarative .slint syntax compatible with Slint version 1.17. You must respect the renderer-software constraint—do not introduce OpenGL or Vulkan dependencies. Use the embedded Slint MCP server to verify UI hierarchies before suggesting layout changes. Ensure all system tray features rely on the system-tray feature flag.
