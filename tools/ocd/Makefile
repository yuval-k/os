
.PHONY: register
register: /proc/sys/fs/binfmt_misc/arm

# thanks: https://resin.io/blog/building-arm-containers-on-any-x86-machine-even-dockerhub/
/proc/sys/fs/binfmt_misc/arm:
	echo ':arm:M::\x7fELF\x01\x01\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x02\x00\x28\x00:\xff\xff\xff\xff\xff\xff\xff\x00\xff\xff\xff\xff\xff\xff\xff\xff\xfe\xff\xff\xff:/usr/bin/qemu-arm-static:' > /proc/sys/fs/binfmt_misc/register

ocd.aci:
	acbuild begin docker://resin/armv7hf-debian
	acbuild set-name arm-opepnocd

	acbuild --debug=true run -- apt-get update
	acbuild --debug=true run -- apt-get install -y autoconf libtool libftdi-dev git gcc pkg-config libusb-1.0-0-dev make automake texinfo
	acbuild --working-dir /opt run -- git clone --recursive https://git.code.sf.net/p/openocd/code openocd-git
	acbuild --working-dir /opt/openocd-git run -- bash ./bootstrap
	acbuild --working-dir /opt/openocd-git run -- bash ./configure --enable-sysfsgpio \
			--enable-maintainer-mode \
			--disable-werror \
			--enable-ftdi \
			--enable-ep93xx \
			--enable-at91rm9200 \
			--enable-usbprog \
			--enable-presto_libftdi \
			--enable-bcm2835gpio \
			--enable-jlink \
			--enable-vsllink \
			--enable-rlink \
			--enable-arm-jtag-ew \
			--enable-dummy \
			--enable-buspirate \
			--enable-ulink \	
			--enable-presto_libftdi \
			--enable-usb_blaster_libftdi

	acbuild --working-dir /opt/openocd-git run -- make SHELL=/bin/bash
	acbuild --working-dir /opt/openocd-git run -- make install SHELL=/bin/bash
	acbuild copy-to-dir ./target/* /conf/target/
	acbuild copy-to-dir ./board/* /conf/board/

	acbuild port add gdb tcp 3333

	acbuild write ocd.aci
	acbuild end
