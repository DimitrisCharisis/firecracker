#!/usr/bin/bash

# A script taken from: https://jvns.ca/blog/2021/01/23/firecracker--start-a-vm-in-less-than-a-second/

set -eu

# Download a kernel and filesystem image
# [ -e hello-vmlinux.bin ] || wget https://s3.amazonaws.com/spec.ccfc.min/img/hello/kernel/hello-vmlinux.bin
# [ -e hello-rootfs.ext4 ] || wget -O hello-rootfs.ext4 https://github.com/firecracker-microvm/firecracker-demo/raw/fea3897ccfab0387ce5cd4fa2dd49d869729d612/xenial.rootfs.ext4
# [ -e hello-id_rsa ] || wget -O hello-id_rsa https://raw.githubusercontent.com/firecracker-microvm/firecracker-demo/ec271b1e5ffc55bd0bf0632d5260e96ed54b5c0c/xenial.rootfs.id_rsa

arch=`uname -m`
dest_kernel="/home/dim/Documents/firecracker/setup_firecracker/vmlinux3_v6_7"
# dest_kernel="/home/dim/Documents/firecracker/resources/x86_64/vmlinux-6.1.102"
# dest_rootfs="hello-rootfs.ext4"
dest_rootfs="/home/dim/Documents/firecracker/setup_firecracker/mycontainer.ext4"
image_bucket_url="https://s3.amazonaws.com/spec.ccfc.min/img/quickstart_guide/$arch"

# if [ ${arch} = "x86_64" ]; then
#     # kernel="${image_bucket_url}/kernels/vmlinux.bin"
#     # rootfs="${image_bucket_url}/rootfs/bionic.rootfs.ext4"
# elif [ ${arch} = "aarch64" ]; then
#     # kernel="${image_bucket_url}/kernels/vmlinux.bin"
#     # rootfs="${image_bucket_url}/rootfs/bionic.rootfs.ext4"
# else
#     echo "Cannot run firecracker on $arch architecture!"
#     exit 1
# fi

# if [ ! -f $dest_kernel ]; then
#     echo "Kernel not found, downloading $kernel..."
#     curl -fsSL -o $dest_kernel $kernel
#     echo "Saved kernel file to $dest_kernel."
# fi

# if [ ! -f $dest_rootfs ]; then
#     echo "Rootfs not found, downloading $rootfs..."
#     curl -fsSL -o $dest_rootfs $rootfs
#     echo "Saved root block device to $dest_rootfs."
# fi

echo "Downloading public key file..."
[ -e hello-id_rsa ] || wget -O hello-id_rsa https://raw.githubusercontent.com/firecracker-microvm/firecracker-demo/ec271b1e5ffc55bd0bf0632d5260e96ed54b5c0c/xenial.rootfs.id_rsa
echo "Saved public key file."

TAP_DEV="fc-88-tap0"

# set up the kernel boot args
MASK_LONG="255.255.255.252"
MASK_SHORT="/30"
FC_IP="169.254.0.21"
TAP_IP="169.254.0.22"
FC_MAC="02:FC:00:00:00:05"

KERNEL_BOOT_ARGS="ro console=ttyS0 nomodules random.trust_cpu=on reboot=k panic=1 pci=off root=/dev/vda"
KERNEL_BOOT_ARGS="${KERNEL_BOOT_ARGS} ip=${FC_IP}::${TAP_IP}:${MASK_LONG}::eth0:off"

# set up a tap network interface for the Firecracker VM
ip link del "$TAP_DEV" 2> /dev/null || true
ip tuntap add dev "$TAP_DEV" mode tap
sysctl -w net.ipv4.conf.${TAP_DEV}.proxy_arp=1 > /dev/null
sysctl -w net.ipv6.conf.${TAP_DEV}.disable_ipv6=1 > /dev/null
ip addr add "${TAP_IP}${MASK_SHORT}" dev "$TAP_DEV"
ip link set dev "$TAP_DEV" up

# make a configuration file
cat <<EOF > vmconfig.json
{
  "boot-source": {
    "kernel_image_path": "$dest_kernel",
    "boot_args": "$KERNEL_BOOT_ARGS"
  },
  "drives": [
    {
      "drive_id": "rootfs",
      "path_on_host": "$dest_rootfs",
      "is_root_device": true,
      "is_read_only": false
    }
  ],
  "network-interfaces": [
      {
          "iface_id": "eth0",
          "guest_mac": "$FC_MAC",
          "host_dev_name": "$TAP_DEV"
      }
  ],
  "machine-config": {
    "vcpu_count": 2,
    "mem_size_mib": 1024,
    "smt": false
  }
}
EOF

/home/dim/Documents/firecracker/build/cargo_target/debug/firecracker --no-api --config-file vmconfig.json
