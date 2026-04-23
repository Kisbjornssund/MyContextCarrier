# MyContextCarrier demo
# Prerequisites: cargo build --release has been run
# Run with: vhs docs/assets/demo.tape

Output docs/assets/demo.gif

Set Shell "zsh"
Set FontSize 14
Set Width 1200
Set Height 650
Set Theme "Dracula"
Set Padding 20
Set TypingSpeed 55ms

Hide
Type "cd ~/NewStelthion/MyContextPort"
Enter
Sleep 200ms
Show

# ── 1. Show what the CLI can do ──────────────────────────────────────────────
Type "./target/release/mycontextport --help"
Enter
Sleep 2500ms

# ── 2. List built-in collectors ──────────────────────────────────────────────
Type "./target/release/mycontextport collector list"
Enter
Sleep 2500ms

# ── 3. Health check ──────────────────────────────────────────────────────────
Type "./target/release/mycontextport collector health shell-history"
Enter
Sleep 2000ms

# ── 4. Scaffold a new collector ──────────────────────────────────────────────
Type "./target/release/mycontextport dev new-collector my-notes"
Enter
Sleep 2000ms
Type "cat collectors/my-notes/collector.py"
Enter
Sleep 3000ms

# ── 5. Start MCP server — this is what Claude Desktop connects to ────────────
Type "# Add to claude_desktop_config.json and Claude knows your context"
Enter
Sleep 1000ms
Type "./target/release/mycontextport mcp serve"
Enter
Sleep 2000ms
