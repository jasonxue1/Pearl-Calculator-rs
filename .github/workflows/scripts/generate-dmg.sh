#!/usr/bin/env bash
set -euo pipefail

version=$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version')

create-dmg \
  --volname "Pearl Calculator" \
  --volicon "dist/Pearl Calculator.app/Contents/Resources/Pearl Calculator.icns" \
  --background "crates/gui/assets/dmg-bg.png" \
  --window-pos 200 120 \
  --window-size 700 400 \
  --icon-size 150 \
  --icon "Pearl Calculator.app" 170 205 \
  --app-drop-link 535 195 \
  --hide-extension "Pearl Calculator.app" \
  --eula "crates/gui/assets/LICENSE.rtf" \
  --no-internet-enable \
  "dist/pearl-calculator-gui_${version}_$1.dmg" \
  "dist/Pearl Calculator.app"
