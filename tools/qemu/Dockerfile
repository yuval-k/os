FROM ubuntu:16.04

RUN apt-get update -y && apt-get install -y curl git build-essential make
RUN git clone git://repo.or.cz/qemu/ar7.git
RUN apt-get install -y python
RUN apt-get install -y pkg-config
RUN apt-get install -y zlib1g-dev
RUN apt-get install -y libglib2.0-dev
RUN apt-get install -y libpixman-1-dev
RUN apt-get install -y libfdt-dev

RUN cd ar7 && git checkout rpi-for-2.8 && ./configure --target-list=arm-softmmu && make -j 2
