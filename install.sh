#!/bin/bash
set -e
apt update -yy
apt install -yy curl screen htop git build-essential
useradd -m topsnek

su - topsnek -c bash <<ENDOFSCRIPT
set -ex
mkdir -p logs

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
. "/home/topsnek/.cargo/env"

mkdir -p .ssh
cat > .ssh/id_rsa <<ENDOFKEY
-----BEGIN OPENSSH PRIVATE KEY-----
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
yjZF/uPY2FbROFAAAACndpbUBtZXJrZWw=
-----END OPENSSH PRIVATE KEY-----
ENDOFKEY
chmod 0600 .ssh/id_rsa

ssh-keyscan github.com > .ssh/known_hosts
git clone git@github.com:wvdschel/topsnek.git
cd topsnek
cargo build --release
ENDOFSCRIPT

cp /home/topsnek/topsnek/target/release/topsnek-server /usr/bin

cat > /etc/systemd/system/topsnek.service <<ENDOFSERVICE
[Unit]
Description=topsnek

[Service]
WorkingDirectory=/home/topsnek/
User=topsnek
ExecStart=/usr/bin/topsnek-server 0.0.0.0

[Install]
WantedBy=multi-user.target
ENDOFSERVICE

systemctl daemon-reload
systemctl enable topsnek.service
systemctl start topsnek.service
