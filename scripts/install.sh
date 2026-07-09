#!/usr/bin/env sh
set -eu

APP_NAME="gate"
SERVICE_NAME="gate-server"
INSTALL_DIR="${GATE_INSTALL_DIR:-/opt/gate}"
CONFIG_DIR="${GATE_CONFIG_DIR:-/etc/gate}"
DATA_DIR="${GATE_DATA_DIR:-/var/lib/gate}"
LOG_DIR="${GATE_LOG_DIR:-/var/log/gate}"
VERSION="${GATE_VERSION:-latest}"
RELEASE_BASE="${GATE_RELEASE_BASE:-https://github.com/Somirk134/Gate/releases/download}"
ARCH="$(uname -m)"
OS_ID=""

need_root() {
  if [ "$(id -u)" -ne 0 ]; then
    echo "Please run as root: sudo sh scripts/install.sh" >&2
    exit 1
  fi
}

detect_os() {
  if [ -r /etc/os-release ]; then
    . /etc/os-release
    OS_ID="${ID:-}"
  fi

  case "$OS_ID" in
    ubuntu|debian|centos|rocky) ;;
    *)
      echo "Unsupported Linux distribution: ${OS_ID:-unknown}" >&2
      exit 1
      ;;
  esac
}

install_dependencies() {
  case "$OS_ID" in
    ubuntu|debian)
      apt-get update
      apt-get install -y ca-certificates curl tar
      ;;
    centos|rocky)
      yum install -y ca-certificates curl tar
      ;;
  esac
}

release_arch() {
  case "$ARCH" in
    x86_64|amd64) echo "x86_64-unknown-linux-gnu" ;;
    aarch64|arm64) echo "aarch64-unknown-linux-gnu" ;;
    *)
      echo "Unsupported CPU architecture: $ARCH" >&2
      exit 1
      ;;
  esac
}

download_release() {
  mkdir -p "$INSTALL_DIR" "$CONFIG_DIR" "$DATA_DIR/certificates" "$LOG_DIR"

  if [ -n "${GATE_RELEASE_URL:-}" ]; then
    RELEASE_URL="$GATE_RELEASE_URL"
  else
    TARGET="$(release_arch)"
    if [ "$VERSION" = "latest" ]; then
      RELEASE_URL="https://github.com/Somirk134/Gate/releases/latest/download/gate-server-${TARGET}.tar.gz"
    else
      RELEASE_URL="${RELEASE_BASE}/${VERSION}/gate-server-${TARGET}.tar.gz"
    fi
  fi

  TMP_ARCHIVE="$(mktemp)"
  echo "Downloading Gate from $RELEASE_URL"
  curl -fL "$RELEASE_URL" -o "$TMP_ARCHIVE"
  tar -xzf "$TMP_ARCHIVE" -C "$INSTALL_DIR"
  rm -f "$TMP_ARCHIVE"

  if [ -f "$INSTALL_DIR/gate-server" ]; then
    chmod 0755 "$INSTALL_DIR/gate-server"
  elif [ -f "$INSTALL_DIR/bin/gate-server" ]; then
    ln -sf "$INSTALL_DIR/bin/gate-server" "$INSTALL_DIR/gate-server"
  else
    echo "Release archive does not contain gate-server" >&2
    exit 1
  fi
}

write_config() {
  if [ ! -f "$CONFIG_DIR/gate.env" ]; then
    TOKEN="${GATE_AUTH_TOKEN:-$(date +%s | sha256sum | awk '{print $1}')}"
    cat > "$CONFIG_DIR/gate.env" <<EOF
GATE_ENV=production
GATE_SERVER_ADDR=0.0.0.0:${GATE_PORT:-5800}
GATE_AUTH_TOKEN=$TOKEN
GATE_DATA_DIR=$DATA_DIR
GATE_CONFIG=$CONFIG_DIR/gate.toml
GATE_CERT_DIR=$DATA_DIR/certificates
GATE_LOG=info
EOF
    chmod 0600 "$CONFIG_DIR/gate.env"
  fi

  if [ ! -f "$CONFIG_DIR/gate.toml" ]; then
    cat > "$CONFIG_DIR/gate.toml" <<EOF
[server]
addr = "0.0.0.0:${GATE_PORT:-5800}"

[storage]
data_dir = "$DATA_DIR"
certificate_dir = "$DATA_DIR/certificates"
EOF
  fi
}

write_service() {
  cat > "/etc/systemd/system/${SERVICE_NAME}.service" <<EOF
[Unit]
Description=Gate Server
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
EnvironmentFile=$CONFIG_DIR/gate.env
ExecStart=$INSTALL_DIR/gate-server
WorkingDirectory=$DATA_DIR
Restart=always
RestartSec=3
User=gate
Group=gate
NoNewPrivileges=true

[Install]
WantedBy=multi-user.target
EOF
}

ensure_user() {
  if ! id gate >/dev/null 2>&1; then
    useradd --system --home-dir "$DATA_DIR" --shell /usr/sbin/nologin gate
  fi
  chown -R gate:gate "$INSTALL_DIR" "$DATA_DIR" "$LOG_DIR"
  chown -R root:gate "$CONFIG_DIR"
}

start_service() {
  systemctl daemon-reload
  systemctl enable "$SERVICE_NAME"
  systemctl restart "$SERVICE_NAME"
  systemctl --no-pager --full status "$SERVICE_NAME" || true
}

need_root
detect_os
install_dependencies
download_release
write_config
ensure_user
write_service
start_service

echo "Gate installed. Config: $CONFIG_DIR/gate.env"
