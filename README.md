# Waveshare RP2040-Zero

## Steps to flash

1. Connect the board to the laptop. 
2. When connecting, press the BOOT button and release once you connect it. 
3. Check the `lsblk` output to see the name of the drive which has been created
4. Create a directory in `/mnt/rp2` using `sudo mkdir /mnt/rp2/`
5. Then mount the created drive into `/mnt/rp2/` using `sudo mount -o uid=$(id -u),gid=$(id -g) /dev/sda1 /mnt/rp2`. Replace `sda1` with the created drive
6. Use the `mkproj` script to create the project (My custom script)
7. Do `cargo run` to upload the program
8. Unmount the drive using `sudo umount /mnt/rp2/`

## Programs

1. [LED Flash]
