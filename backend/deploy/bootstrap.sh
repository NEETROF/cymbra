#!/usr/bin/env bash
# Idempotent provisioner for the Cymbra single-box prod. Re-run safe.
#
# Reconstructs the box's SHAPE so a lost box is ~15 min to rebuild, not hours of
# scrollback: system hardening (ufw, key-only SSH on a custom port, fail2ban,
# unattended-upgrades), Docker, the /opt/cymbra dirs, executable deploy scripts,
# and the nightly backup cron skeleton.
#
# SECRET-FREE by design — this file is committed. It NEVER contains DB passwords,
# the token signing key, S3 creds, PATs, or the box IP/domain. Everything sensitive
# is either a parameter (env var) or restored separately AFTER this runs:
#   * app secrets   → drop the vaulted .env at /opt/cymbra/backend/deploy/.env (chmod 600)
#   * S3 backup creds → fill the empty /etc/cymbra/backup.env this creates
#   * GHCR pull auth → `docker login ghcr.io -u <user>` once
#   * DB data        → restore a dump (see deploy/backup.sh header)
#   * TLS + app wiring → first launch (DEPLOY.md §5) + deploy.sh <version>
#
# Parameters (env vars — nothing box-specific is hardcoded):
#   SSH_PORT       SSH port to run on. Default "22" (no move). Set e.g. 49222 to
#                  move SSH off 22 (socket-activation aware, IPv4+IPv6 explicit).
#   DEPLOY_PUBKEY  Full public-key LINE for the CI deploy key. If set, installed
#                  with a forced-command (deploy only, no shell). A public key is
#                  NOT a secret, but it is keypair-specific so it stays a parameter.
#   TARGET_USER    Unprivileged owner user. Default: ${SUDO_USER:-ubuntu}.
#
# Usage on a fresh box (repo's backend/ present at /opt/cymbra/backend via git/scp):
#   SSH_PORT=49222 DEPLOY_PUBKEY="ssh-ed25519 AAAA... cymbra-ci-deploy" \
#     sudo -E bash /opt/cymbra/backend/deploy/bootstrap.sh
#
# After it finishes: restore .env from your vault, `docker login ghcr.io`, first
# launch, then restore the latest DB dump. If SSH_PORT != 22, reconnect on the new
# port. See DEPLOY.md.
set -euo pipefail

[[ $EUID -eq 0 ]] || { echo "run with sudo/root" >&2; exit 1; }

SSH_PORT="${SSH_PORT:-22}"
TARGET_USER="${TARGET_USER:-${SUDO_USER:-ubuntu}}"
DEPLOY_PUBKEY="${DEPLOY_PUBKEY:-}"
DEPLOY_DIR="/opt/cymbra/backend/deploy"
id "$TARGET_USER" >/dev/null 2>&1 || { echo "TARGET_USER '$TARGET_USER' does not exist" >&2; exit 1; }
[[ "$SSH_PORT" =~ ^[0-9]+$ ]] || { echo "SSH_PORT must be numeric" >&2; exit 1; }
log() { printf '\n=== %s ===\n' "$*"; }

log "1. Packages (idempotent)"
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
apt-get -y -qq install ufw fail2ban unattended-upgrades ca-certificates curl gnupg \
                       docker.io docker-compose-v2 awscli

log "2. Docker enabled + $TARGET_USER in docker group"
systemctl enable --now docker
usermod -aG docker "$TARGET_USER"

log "3. /opt/cymbra dirs + executable deploy scripts"
mkdir -p "$DEPLOY_DIR" /opt/cymbra/backend/db/init
chown -R "$TARGET_USER:$TARGET_USER" /opt/cymbra
for s in backup.sh deploy.sh deploy-forced.sh bootstrap.sh; do
  [[ -f "$DEPLOY_DIR/$s" ]] && chmod +x "$DEPLOY_DIR/$s"
done

log "4. Firewall (open SSH_PORT/80/443 BEFORE enabling — no lockout)"
ufw allow "$SSH_PORT"/tcp comment 'SSH'
ufw allow 80/tcp  comment 'HTTP (ACME + redirect)'
ufw allow 443/tcp comment 'HTTPS gRPC+JWKS'
ufw default deny incoming
ufw default allow outgoing
ufw --force enable

log "5. SSH hardening (key-only)"
cat > /etc/ssh/sshd_config.d/99-cymbra-hardening.conf <<'EOF'
PasswordAuthentication no
PubkeyAuthentication yes
PermitRootLogin prohibit-password
KbdInteractiveAuthentication no
EOF
sshd -t

if [[ "$SSH_PORT" != "22" ]] && systemctl is-active --quiet ssh.socket; then
  log "5b. Move SSH to $SSH_PORT (socket-activated; bind IPv4+IPv6 explicitly)"
  mkdir -p /etc/systemd/system/ssh.socket.d
  cat > /etc/systemd/system/ssh.socket.d/override.conf <<EOF
[Socket]
ListenStream=
ListenStream=0.0.0.0:$SSH_PORT
ListenStream=[::]:$SSH_PORT
EOF
  systemctl daemon-reload
  systemctl restart ssh.socket
  echo "SSH now listens on $SSH_PORT only — reconnect there after this run."
else
  systemctl restart ssh
fi

log "6. fail2ban (sshd jail on $SSH_PORT, systemd backend)"
cat > /etc/fail2ban/jail.local <<EOF
[DEFAULT]
bantime  = 1h
findtime = 10m
maxretry = 5
backend  = systemd

[sshd]
enabled = true
port    = $SSH_PORT
EOF
systemctl enable --now fail2ban
systemctl restart fail2ban

log "7. Unattended security upgrades"
cat > /etc/apt/apt.conf.d/20auto-upgrades <<'EOF'
APT::Periodic::Update-Package-Lists "1";
APT::Periodic::Unattended-Upgrade "1";
EOF
systemctl enable --now unattended-upgrades

if [[ -n "$DEPLOY_PUBKEY" ]]; then
  log "8. CI deploy key (forced-command: deploy only, no shell)"
  AK="/home/$TARGET_USER/.ssh/authorized_keys"
  install -d -m 700 -o "$TARGET_USER" -g "$TARGET_USER" "/home/$TARGET_USER/.ssh"
  touch "$AK"; chown "$TARGET_USER:$TARGET_USER" "$AK"; chmod 600 "$AK"
  KEYID="$(awk '{print $NF}' <<<"$DEPLOY_PUBKEY")"
  if grep -qF "$KEYID" "$AK"; then
    echo "deploy key already present"
  else
    printf 'command="%s/deploy-forced.sh",no-agent-forwarding,no-port-forwarding,no-X11-forwarding,no-pty %s\n' \
      "$DEPLOY_DIR" "$DEPLOY_PUBKEY" >> "$AK"
    echo "deploy key installed"
  fi
else
  echo "(no DEPLOY_PUBKEY given — skipping CI deploy key)"
fi

log "9. Nightly backup cron + secret-free env skeleton"
mkdir -p /etc/cymbra
if [[ ! -f /etc/cymbra/backup.env ]]; then
  cat > /etc/cymbra/backup.env <<'EOF'
# Sourced by the nightly backup cron. chmod 600, root. Fill the S3 block to enable
# the off-box copy (leave empty for local-only backups). NO secrets are committed —
# these are set by the operator on the box.
export BACKUP_DIR=/var/backups/cymbra
export BACKUP_RETENTION_DAYS=14
export S3_ENDPOINT=
export S3_BUCKET=
export AWS_DEFAULT_REGION=
export AWS_ACCESS_KEY_ID=
export AWS_SECRET_ACCESS_KEY=
# Encrypts the .env off-box (redundancy for your password-manager vault). Generate a
# strong one — e.g. `openssl rand -base64 32` — and store it in your vault too (a lost
# box means restoring this passphrase from the vault to decrypt the S3 copy). Empty =
# skip the encrypted .env upload.
export BACKUP_ENV_PASSPHRASE=
EOF
  chmod 600 /etc/cymbra/backup.env
  echo "created /etc/cymbra/backup.env (empty S3 — fill to enable off-box copy)"
else
  echo "/etc/cymbra/backup.env already exists — left untouched"
fi
touch /var/log/cymbra-backup.log && chmod 640 /var/log/cymbra-backup.log
cat > /etc/logrotate.d/cymbra-backup <<'EOF'
/var/log/cymbra-backup.log {
  weekly
  rotate 8
  compress
  missingok
  notifempty
}
EOF
CRON_LINE=". /etc/cymbra/backup.env; $DEPLOY_DIR/backup.sh >> /var/log/cymbra-backup.log 2>&1"
CRON_ENTRY="0 3 * * * $CRON_LINE"
( crontab -l 2>/dev/null | grep -v 'backend/deploy/backup.sh' || true; printf '%s\n' "$CRON_ENTRY" ) | crontab -
systemctl enable --now cron 2>/dev/null || true

log "DONE"
cat <<EOF
Next (secrets restored OUT of this script):
  1. Restore the vaulted .env  -> $DEPLOY_DIR/.env   (chmod 600)
  2. docker login ghcr.io -u <github-user>
  3. First launch (DEPLOY.md §5) or ./deploy.sh <version>
  4. Restore the latest DB dump (deploy/backup.sh header) if rebuilding
$( [[ "$SSH_PORT" != "22" ]] && echo "  * Reconnect on SSH port $SSH_PORT" || true )
EOF
