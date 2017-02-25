#!/bin/sh

sudo rkt run --net=host --volume tftproot,kind=host,source=/home/yuval/Projects/os/tftproot,readOnly=true coreos.com/dnsmasq:v0.3.0 --mount volume=tftproot,target=/tftpboot -- --port=0 --dhcp-range=192.168.1.255,proxy --log-dhcp --enable-tftp --tftp-root=/tftpboot --pxe-service=0,"Raspberry Pi Boot" --interface=wlp2s0