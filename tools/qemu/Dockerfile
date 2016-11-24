FROM ubuntu:16.04

RUN apt-get update -y && apt-get install -y curl git build-essential make python pkg-config zlib1g-dev libglib2.0-dev libpixman-1-dev libfdt-dev gdb
# RUN git clone git://repo.or.cz/qemu/ar7.git qemu && cd qemu && git checkout rpi-for-2.8 
RUN git clone https://github.com/qemu/qemu qemu

RUN cd qemu && ./configure --enable-debug-info --enable-debug --disable-pie --target-list=arm-softmmu && make -j 2 && make install
