# About
This program is meant to replicate the functionality of the original `DeepCool Digital`
Windows program and I am gradually adding support to new devices.

If you have a device that has not been added or tested yet, please read the notes below the
supported devices.
If you think you can collaborate, please write an issue so we can get in touch.

# Installation
Simply download the latest [release](https://github.com/Nortank12/deepcool-digital-linux/releases)
and run it in the command line. You will need root permission to send data to the device.

> [!TIP]
> On AMD's Zen architecture CPUs, you can install the [zenpower3](https://github.com/PutinVladimir/zenpower3)
> driver, to have a more accurate reading of the CPU die.

### Rootless mode (optional)
If you need to run the program without root privilege, you can create a `udev` rule to access all necessary resources as a user.

1. Locate your directory, it can be `/lib/udev/rules.d` or `/etc/udev/rules.d`.
```bash
cd /lib/udev/rules.d
```
2. Create a new file called `99-deepcool-digital.rules`.
```bash
sudo nano 99-deepcool-digital.rules
```
3. Copy the contents:
```bash
# Intel RAPL energy usage file
ACTION=="add", SUBSYSTEM=="powercap", KERNEL=="intel-rapl:0", RUN+="/bin/chmod 444 /sys/class/powercap/intel-rapl/intel-rapl:0/energy_uj"

# DeepCool HID raw devices
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3633", MODE="0666"
```
4. Reboot your computer.

# Supported devices

### CPU Coolers
<table>
    <tr>
        <th>Name</th>
        <th>Supported</th>
    </tr>
    <tr>
        <td>AG400 DIGITAL</td>
        <td align="center">✅</td>
    </tr>
    <tr>
        <td>AG620 DIGITAL</td>
        <td align="center">✅</td>
    </tr>
    <tr>
        <td>AK400 DIGITAL</td>
        <td align="center">✅</td>
    </tr>
    <tr>
        <td>AK500 DIGITAL</td>
        <td align="center">✅</td>
    </tr>
    <tr>
        <td>AK500S DIGITAL</td>
        <td align="center">✅</td>
    </tr>
    <tr>
        <td>AK620 DIGITAL</td>
        <td align="center">✅</td>
    </tr>
    <tr>
        <td>LD240</td>
        <td align="center">✅</td>
    </tr>
    <tr>
        <td>LD360</td>
        <td align="center">✅</td>
    </tr>
</table>

### Cases
<table>
    <tr>
        <th>Name</th>
        <th>Supported</th>
    </tr>
    <tr>
        <td>CH360 DIGITAL</td>
        <td align="center">⚠️</td>
    </tr>
    <tr>
        <td>CH560 DIGITAL</td>
        <td align="center">✅</td>
    </tr>
    <tr>
        <td>MORPHEUS</td>
        <td align="center">✅</td>
    </tr>
</table>

**✅: Fully supported &nbsp; ⚠️: Not tested**

> [!IMPORTANT]
> - If your device is not on the list, you can still run the program and see if it detects it.
> - If your device is on the list but untested, please try to check all the features to see if they work as expected.
>
> In any case, you can create an issue or add a comment to an existing one.

# Usage
You can run the program with or without providing any options.
```bash
sudo ./deepcool-digital-linux [OPTIONS]
```
```
Options:
  -m, --mode <MODE>  Change the display mode between "temp, usage, auto" [default: temp]
      --pid <ID>     Specify the Product ID if you use mutiple devices
  -f, --fahrenheit   Change temperature unit to Fahrenheit
  -a, --alarm        Enable the alarm [85˚C | 185˚F]

Commands:
  -l, --list         Print Product ID of the connected devices
  -h, --help         Print help
  -v, --version      Print version
```

### Using multiple devices (optional)
If you have multiple devices connected, you can run the following
command to detect them:
```bash
sudo ./deepcool-digital-linux --list
```
```
Device list [PID | Name]
-----
4 | AK500S-DIGITAL
7 | MORPHEUS
```
After identifying, you can run them separately by providing their Product ID:
```bash
sudo ./deepcool-digital-linux --pid 4
```
```bash
sudo ./deepcool-digital-linux --pid 7
```
If you want to run them automatically, you can create 2 services
instead of 1.

For example:
- `deepcool-digital-case.service`
- `deepcool-digital-cooler.service`

# Automatic start

## Systemd (Arch, Debian, Ubuntu, Fedora, etc.)
1. Copy the `deepcool-digital-linux` to the `/usr/sbin/` folder.
```bash
sudo cp ./deepcool-digital-linux /usr/sbin/
```
2. Create the service file in the `/etc/systemd/system/` folder.
```bash
sudo nano /etc/systemd/system/deepcool-digital.service
```
3. Copy the contents:
```properties
[Unit]
Description=DeepCool Digital

[Service]
ExecStart=/usr/sbin/deepcool-digital-linux # arguments here

[Install]
WantedBy=multi-user.target
```
4. Enable the service
```bash
sudo systemctl enable deepcool-digital
```
*Note: The program will run automatically after the next boot.*

## OpenRC (Gentoo, Artix Linux, etc.)
1. Copy the `deepcool-digital-linux` to the `/usr/sbin/` folder.
```bash
sudo cp ./deepcool-digital-linux /usr/sbin/
```
2. Create the service file in the `/etc/init.d/` folder.
```bash
sudo nano /etc/init.d/deepcool-digital
```
3. Copy the contents:
```properties
#!/sbin/openrc-run

description="DeepCool Digital"
command="/usr/sbin/deepcool-digital-linux"
command_args="" # arguments here
command_background=1
pidfile="/run/deepcool-digital.pid"
```
4. Allow execution on the service file
```bash
sudo chmod +x /etc/init.d/deepcool-digital
```
5. Enable the service
```bash
sudo rc-update add deepcool-digital default
```
*Note: The program will run automatically after the next boot.*

# More information
[Device List and USB Mapping Tables](device-list)

# Development
LD Series: [asdfzdfj](https://github.com/asdfzdfj) / [deepcool-ld-digital-hidapi](https://github.com/asdfzdfj/deepcool-ld-digital-hidapi)
