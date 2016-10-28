FROM ubuntu:16.04

RUN apt-get update -y && apt-get install -y curl git build-essential make python pkg-config zlib1g-dev libglib2.0-dev libpixman-1-dev libfdt-dev
RUN git clone git://repo.or.cz/qemu/ar7.git

RUN cd ar7 && git checkout rpi-for-2.8 && ./configure --target-list=arm-softmmu && make -j 2 && make install
