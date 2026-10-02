#!/bin/sh
# Show the first differing output line between CPython and pyjs.
f=$1
python3 "$f" > /tmp/fd_py.txt 2>&1
node "$(dirname "$0")/../dist/src/cli.js" "$f" > /tmp/fd_js.txt 2>&1
diff /tmp/fd_py.txt /tmp/fd_js.txt | head -${2:-6}
