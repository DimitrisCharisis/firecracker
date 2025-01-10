#!/bin/bash

set -xe

CONTAINER_ID=$(docker ps -a -q -f name=setup-container2)

MOUNTDIR=mnt
FS=mycontainer.ext4

mkdir -p $MOUNTDIR
qemu-img create -f raw $FS 800M
mkfs.ext4 $FS
mount -t ext4 $FS $MOUNTDIR

docker export $CONTAINER_ID | tar -C $MOUNTDIR -xvf -

umount $MOUNTDIR
