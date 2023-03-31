x() { ( set -x; "$@"; ); }
i() { echo "[1;36mInfo:[m $*"; }
e() { echo "[1;31mError:[m $*"; exit 1; }
