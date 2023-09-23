#!/bin/bash
CUR_DIR=$(dirname $0)

dd if=/dev/zero of="$CUR_DIR/myimage.img" bs=1M count=2