#!/bin/bash
APP_TITLE="BashIt"
APP_CONTENT="Bash command file curator and profile manager.

• Open any bash history or script file to import it as a profile
• Lock commands as residents — they survive any clean operation
• Switch between profiles for different tasks or workflows
• Export the active profile cleanly to ~/.bash_history
• Close open terminals after a clean to fully apply changes"

FLAG="$HOME/.config/lean/bashit-welcomed"
mkdir -p "$(dirname "$FLAG")"

zenity --info \
    --title="Welcome to $APP_TITLE" \
    --text="$APP_CONTENT" \
    --width=400 2>/dev/null || true

touch "$FLAG"
