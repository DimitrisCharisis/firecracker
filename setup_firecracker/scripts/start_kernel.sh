#!/bin/bash

set -xe

KERNEL_IMAGE="/home/dim/Documents/firecracker/setup_firecracker/vmlinux3_v6_7"
DISK_IMAGE="/home/dim/Documents/firecracker/setup_firecracker/mycontainer.ext4"

# Set the kernel
sudo curl --unix-socket /tmp/firecracker.socket -i \
-X PUT 'http://localhost/boot-source' \
-H 'Accept: application/json' \
-H 'Content-Type: application/json' \
-d "{
\"kernel_image_path\": \"$KERNEL_IMAGE\",
\"boot_args\": \"console=ttyS0 reboot=k panic=1 pci=off 'root=/dev/vda'\"
}"

# Set the root filesystem
sudo curl --unix-socket /tmp/firecracker.socket -i \
-X PUT 'http://localhost/drives/rootfs' \
-H 'Accept: application/json' \
-H 'Content-Type: application/json' \
-d "{
\"drive_id\": \"rootfs\",
\"path_on_host\": \"$DISK_IMAGE\",
\"is_root_device\": true,
\"is_read_only\": false
}"

# Start the instance
sudo curl --unix-socket /tmp/firecracker.socket -i \
-X PUT "http://localhost/actions" \
-d '{ "action_type": "InstanceStart" }'
