# systemd-boot configuration generator
This project attempts to create the .conf files located in /boot/loader/entries/. It will read two files:

- /etc/os-release - for OS information
- /etc/default/cmdline - kernel parameters

## Configuration
/etc/default/cmdline contains one variable: CMDLINE="", the value of which must be quoted, these are the parameters passed to the kernel at boot.

## Installation
    cargo install systemd-boot-gen
