#!/usr/bin/bash

set -e

echo "Upgrading pi-extender service..."

systemctl stop pi-extender.service
systemctl daemon-reload
systemctl enable pi-extender.service
systemctl start pi-extender.service

echo "Done."

exit 0
