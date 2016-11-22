Code is based these tutorials:

- http://wiki.osdev.org/ARM_Integrator-CP_Bare_Bones
- http://os.phil-opp.com/
- http://antoinealb.net/programming/2015/05/01/rust-on-arm-microcontroller.html
- http://blogs.bu.edu/md/2011/11/15/the-dark-art-of-linker-scripts/
- https://github.com/mrd/puppy/

To build a cross compiler for OSX, combine these guides to something working:
note that it's ok that the c library fails to compile as we only need the assembler and linker that were compiled before (so as long as it fails after compiling basic gcc we are good)

- http://crosstool-ng.org/
- http://crosstool-ng.org/hg/crosstool-ng/file/715b711da3ab/docs/MacOS-X.txt
- https://www.zephyrproject.org/doc/getting_started/installation_mac.html
- https://github.com/crosstool-ng/crosstool-ng/issues/211
- http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.faqs/ka14041.html

My config is `ct-ng.config`. copy it to .config and build (on macos create 4gb, case sensitive disk volume fist).
you can then build cross platform gcc:
```
ct-ng arm-unknown-linux-gnueabi
ct-ng build
export PATH="${PATH}:${PWD}/.build/arm-unknown-linux-gnueabi/bin"
arm-unknown-linux-gnueabi-gcc
see http://crosstool-ng.org/hg/crosstool-ng/file/715b711da3ab/docs/MacOS-X.txt about creating case sensistive disk image
```
This will most likely to fail. as long as it failed after building the assembler and linker, you are good. (I use .build folder because my build failed in the middle and never installed.)

I called my volume Crosstool, so i build it with:

```
export PATH=/Volumes/Crosstool/.build/armv7-unknown-linux-gnueabihf/buildtools/bin/:/Volumes/Crosstool/.build/arm-unknown-linux-gnueabi/buildtools/bin/:$PATH
make toolchain # do this just once
make kernel.img
make emulate
```


Stub:
builds page table - identity page table for itself and devices, and proper virtual table for the kernel.
kernel will later remove the stub and remap memory

# Resources:
https://qemu.weilnetz.de/test/system/arm/raspberry-pi/ | http://repo.or.cz/w/qemu/ar7.git/
https://github.com/0xabu/qemu
http://wiki.osdev.org/ARM_Integrator-CP_IRQTimerAndPICAndTaskSwitch
http://wiki.osdev.org/User:Pancakes/arm_qemu_realview-pb-a
http://www.cl.cam.ac.uk/projects/raspberrypi/tutorials/os/
https://github.com/rsta2/circle

http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dai0425/ch04s07s01.html

migration: http://infocenter.arm.com/help/index.jsp?topic=/com.arm.doc.dai0425/BABCDDJG.html
https://github.com/genodelabs/genode
https://github.com/RIOT-OS/RIOT
http://stackoverflow.com/questions/28610045/how-to-start-additional-cores-on-the-raspberry-pi-2-under-bare-metal

princeton os course:
https://www.cs.princeton.edu/courses/archive/fall10/cos318/index.html
https://www.cs.princeton.edu/courses/archive/fall10/cos318/lectures/MutexImplementation.pdf
http://kos.enix.org/pub/how_to_bring_smp.html