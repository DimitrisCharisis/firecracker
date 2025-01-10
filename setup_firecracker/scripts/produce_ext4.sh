#!/bin/bash

set -xe

IMG_ID=$(docker build -q .)
# CONTAINER_ID=$(docker run -td $IMG_ID /bin/bash)
CONTAINER_ID=$(docker ps -a -q -f name=setup-container)

MOUNTDIR=mnt
FS=mycontainer.ext4

mkdir $MOUNTDIR
qemu-img create -f raw $FS 800M
mkfs.ext4 $FS
mount $FS $MOUNTDIR
docker cp $CONTAINER_ID:/ $MOUNTDIR
umount $MOUNTDIR
