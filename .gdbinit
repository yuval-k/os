set substitute-path /buildslave/rust-buildbot/slave/nightly-dist-rustc-cross-host-linux/build/src /home/yuval/.multirust/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/src

symbol-file ./target/kernel.elf

# target remote localhost:1234
target remote 192.168.1.20:3333
continue

