x() { ( set -x; "$@"; ); }
i() { echo "[1;36mInfo:[m $*"; }
d() { echo "[1;32mDone:[m $*"; exit 0; }
e() { echo "[1;31mError:[m $*"; exit 1; }
