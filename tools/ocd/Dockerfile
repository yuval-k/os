FROM armhfbuild/debian

RUN apt-get update && apt-get install -y autoconf libtool libftdi-dev git gcc pkg-config libusb-1.0-0-dev make

RUN cd /opt/ && \
    git clone --recursive git://git.code.sf.net/p/openocd/code openocd-git

RUN cd /opt/openocd-git && \
    ./bootstrap && \
    ./configure --enable-sysfsgpio \
         --enable-maintainer-mode \
         --disable-werror \
          --enable-ftdi \
         --enable-ep93xx \
         --enable-at91rm9200 \
         --enable-usbprog \
         --enable-presto_libftdi \
         --enable-jlink \
         --enable-vsllink \
         --enable-rlink \
         --enable-arm-jtag-ew \
         --enable-dummy \
         --enable-buspirate \
         --enable-ulink \
         --enable-presto_libftdi \
         --enable-usb_blaster_libftdi \
         --prefix=/usr \
    && \
    make && make install

RUN  cd /opt/openocd-git && \
     cp -r tcl/ /usr/share/openocd/ && \
     cp /tmp/raspberrypi-due.tcl /usr/share/openocd/tcl/board/


# RUN cd /tmp/ && wget https://gist.github.com/giseburt/e832ed40e3c77fcf7533/raw/e8c71233970e4d42eed7c3bf4b13390cdcf2a1fd/raspberrypi-due.tcl


COPY board/raspberrypi-raspberrypi.tcl /usr/share/openocd/tcl/board/
COPY target/raspberry.cfg /usr/share/openocd/tcl/target/
EXPOSE 3333