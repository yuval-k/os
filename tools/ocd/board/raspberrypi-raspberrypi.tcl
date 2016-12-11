source [find interface/sysfsgpio-raspberrypi.cfg]
set _CHIPNAME sam3

source [find target/raspberry.cfg]
# $_TARGETNAME configure -event gdb-attach { reset init }
