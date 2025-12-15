#!/bin/bash

echo "=== Testing xdotool functionality ==="

echo "1. Testing xdotool installation:"
which xdotool
if [ $? -eq 0 ]; then
    echo "✅ xdotool found at: $(which xdotool)"
else
    echo "❌ xdotool not found"
    echo "Please install with: sudo apt install xdotool"
    exit 1
fi

echo ""
echo "2. Testing xdotool version:"
xdotool --version

echo ""
echo "3. Testing basic key input:"
echo "This test will type 'TEST123' after 3 seconds..."
sleep 3

echo "4. Please click in a text editor (gedit, vim, terminal, etc.) NOW"
sleep 2

echo "5. Typing test text..."
xdotool type "TEST123"

echo ""
echo "6. Testing F4 key:"
xdotool key F4

echo ""
echo "=== Test completed ==="
echo "If you see 'TEST123' typed and the F4 key was pressed, xdotool is working correctly."