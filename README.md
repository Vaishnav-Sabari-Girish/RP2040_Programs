# Waveshare RP2040-Zero

## Steps to flash

1. Connect the board to the laptop.
2. When connecting, press the BOOT button and release once you connect it.
3. Check the `lsblk` output to see the name of the drive which has been created
4. Create a directory in `/mnt/rp2` using `sudo mkdir /mnt/rp2/`
5. Then mount the created drive into `/mnt/rp2/` using
   `sudo mount -o uid=$(id -u),gid=$(id -g) /dev/sda1 /mnt/rp2`. Replace `sda1`
   with the created drive
6. Use the `mkproj` script to create the project (My custom script)
7. Do `cargo run` to upload the program
8. Unmount the drive using `sudo umount /mnt/rp2/`

### Alternative way

1. Connect the board to the laptop
2. To put it in BOOT mode press the BOOT button and the RESET button and release
   the RESET button after a few seconds while pressing the BOOT button.
3. It'll go into BOOT mode.
4. Run `just run` to flash the program

## Programs

1. [LED Flash](./led_flash/src/main.rs)
2. [Serial Monitor](./uart_serial_monitor/src/main.rs)
3. [Internal Temperature Sensor](./internal_temp_sensor/src/main.rs)
4. [E-Paper Display](./e-paper-display/src/main.rs)
5. [E-Paper Display `ratatui`](./e-paper-display-ratatui/src/main.rs)
6. [E-Paper Display `tui-big-text`](./e-paper-big-text/src/main.rs)
7. [E-Paper Display `ascii` text](./e-paper-display-ascii/src/main.rs)
