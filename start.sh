#!/bin/bash
cd "$(dirname "$0")"
echo "Chaser starting at http://localhost:8765"
echo "Press Ctrl+C to stop"
python3 -m http.server 8765
