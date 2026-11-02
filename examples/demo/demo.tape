# Renders examples/demo.gif. Run from the repo root with `make demo`.

Output examples/demo.gif

Require bash
Require nvim

Set Shell bash
Set FontSize 20
Set Width 1400
Set Height 820
Set Padding 24
Set Margin 16
Set MarginFill "#14161b"
Set BorderRadius 10
Set WindowBar Colorful
Set TypingSpeed 55ms
Set CursorBlink false

# Scratch directory with an untidy Quadlet file for the formatting scene
# and a broken unit for the CLI scene.
Hide
Type "export REPO=$PWD SYSTEMD_LSP_BIN=$PWD/target/release/systemd-lsp PS1='\[\e[1;35m\]❯\[\e[0m\] '"
Enter
Type "alias nvim='nvim -u $REPO/examples/demo/init.lua' systemd-lsp=$SYSTEMD_LSP_BIN"
Enter
Type "cd $(mktemp -d) && printf '[Container]\nImage =  docker.io/library/nginx:latest\nPublishPort=   8080:80\n\n\n\nVolume=web-data.volume:/usr/share/nginx/html\n[Install]\nWantedBy   =default.target' > web.container && printf '[Unit]\nDescription=Legacy worker\nRequiers=network.target\n\n[Service]\nExecStart=\n\n[Instal]\nWantedBy=multi-user.target\n' > legacy.service && clear"
Enter
Show

Sleep 500ms
Type "nvim checkout.service"
Enter
Sleep 1.5s

# 1. Directive and value completion.
Type "i[Unit]"
Escape
Type "o"
Type "Desc"
Sleep 1.2s
Enter
Type "Checkout API"
Escape
Type "o"
Type "After=network-online.target"
Escape
Type "o"
Enter
Type "[Service]"
Escape
Type "o"
Type "Type="
Sleep 1.5s
Type "no"
Sleep 800ms
Enter
Escape
Type "o"
Type "ExecStart=/usr/bin/checkout --port 8080"
Escape
Type "o"

# 2. Diagnostics as you type.
Type "Restart=sometimes"
Escape
Sleep 2.5s
Type "0f=lC"
Type "on-f"
Sleep 1.2s
Enter
Escape
Sleep 1.5s

# 3. Hover documentation.
Type "6G0"
Sleep 300ms
Type "K"
Sleep 3.5s
Escape
Type ":wq"
Sleep 300ms
Enter
Sleep 500ms

# 4. Podman Quadlet support and formatting.
Type "nvim web.container"
Enter
Sleep 2s
Type ":lua vim.lsp.buf.format()"
Sleep 500ms
Enter
Sleep 2.5s
Type ":wq"
Enter
Sleep 500ms

# 5. CLI mode for linting in CI.
Type "systemd-lsp *.service"
Sleep 300ms
Enter
Sleep 4s
