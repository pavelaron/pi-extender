#!/usr/bin/bash

set -e

echo "Creating pi-extender service..."

cp ./pi-extender.service /etc/systemd/system/pi-extender.service

systemctl daemon-reload
systemctl enable pi-extender.service

echo "Starting pi-extender service..."
systemctl start pi-extender.service

echo "Done."

exit 0
