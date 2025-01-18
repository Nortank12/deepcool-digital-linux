# Table of Contents
- [About](#about)
- [Installation](#installation)
- [Supported Devices](#supported-devices)
    - [MYSTIQUE Series](#mystique-series)
- [Usage](#usage)
- [Automatic Start](#automatic-start)
    - [Systemd](#systemd-arch-debian-ubuntu-fedora-etc)
    - [OpenRC](#openrc-gentoo-artix-linux-etc)
- [Building from Source](#building-from-source)
- [Device List](#more-information)

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

### Rootless Mode <sup>(optional)</sup>
If you need to run the program without root privilege, you can create a `udev` rule to access all necessary resources as a user.

1. Locate your directory, it can be `/lib/udev/rules.d` or `/etc/udev/rules.d`
```bash
cd /lib/udev/rules.d
```
2. Create a new file called `99-deepcool-digital.rules`
```bash
sudo nano 99-deepcool-digital.rules
```
3. Insert the following:
```bash
# Intel RAPL energy usage file
ACTION=="add", SUBSYSTEM=="powercap", KERNEL=="intel-rapl:0", RUN+="/bin/chmod 444 /sys/class/powercap/intel-rapl/intel-rapl:0/energy_uj"

# DeepCool HID raw devices
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3633", MODE="0666"

# CH510 MESH DIGITAL
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="34d3", ATTRS{idProduct}=="1100", MODE="0666"
```
4. Reboot your computer

<details>
<summary><b>Steps for NixOS</b></summary>

1. Locate and edit your `configuration.nix` file
```bash
sudo nano /etc/nixos/configuration.nix
```
2. Insert the following:
```nix
  services.udev.extraRules = ''
    # Intel RAPL energy usage file
    ACTION=="add", SUBSYSTEM=="powercap", KERNEL=="intel-rapl:0", RUN+="${pkgs.coreutils}/bin/chmod 444 /sys/class/powercap/intel-rapl/intel-rapl:0/energy_uj"

    # DeepCool HID raw devices
    SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3633", MODE="0666"

    # CH510 MESH DIGITAL
    SUBSYSTEM=="hidraw", ATTRS{idVendor}=="34d3", ATTRS{idProduct}=="1100", MODE="0666"
  '';
```
3. Rebuild your system
```bash
sudo nixos-rebuild switch
```
4. Reboot your computer
</details>

# Supported Devices

### CPU Air Coolers
<table>
    <tr>
        <th>Name</th>
        <th>Supported</th>
    </tr>
    <tr>
        <td>AG300 DIGITAL</td>
        <td align="center">✅</td>
    </tr>
    <tr>
        <td>AG400 DIGITAL</td>
        <td align="center">✅</td>
    </tr>
    <tr>
        <td>AG500 DIGITAL</td>
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
        <td>AK400 DIGITAL PRO</td>
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
        <td>AK620 DIGITAL PRO</td>
        <td align="center">✅</td>
    </tr>
</table>

### CPU Liquid Coolers
<table>
    <tr>
        <th>Name</th>
        <th>Supported</th>
    </tr>
    <tr>
        <td>LD240</td>
        <td align="center">✅</td>
    </tr>
    <tr>
        <td>LD360</td>
        <td align="center">✅</td>
    </tr>
    <tr>
        <td>LP240</td>
        <td align="center">⚠️</td>
    </tr>
    <tr>
        <td>LP360</td>
        <td align="center">⚠️</td>
    </tr>
    <tr>
        <td>LS520 SE DIGITAL</td>
        <td align="center">✅</td>
    </tr>
    <tr>
        <td>LS720 SE DIGITAL</td>
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
        <td>CH170 DIGITAL</td>
        <td align="center">✔️</td>
    </tr>
    <tr>
        <td>CH360 DIGITAL</td>
        <td align="center">✅</td>
    </tr>
    <tr>
        <td>CH510 MESH DIGITAL</td>
        <td align="center">✅</td>
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

**✅: Fully supported**

**✔️: Partially supported**<br>
*Some display modes are unavailable due to resource limitations.*

**⚠️: Not tested &nbsp; ❓: Not added**

> [!IMPORTANT]
> - If your device is not added yet, you can still run the program and see if it detects it.
> - If your device is not tested, please try to check all the features to see if they work as expected.
>
> In any case, you can create an issue or add a comment to an existing one.

### MYSTIQUE Series
These devices are unique since they have an LCD display, and I do not personally own one. However, DeadSurfer opened a [discussion](https://github.com/Nortank12/deepcool-digital-linux/discussions/18) and if you can figure out how to make it work, you can share it there or create a pull request.

# Usage
You can run the program with or without providing any options.
```bash
sudo ./deepcool-digital-linux [OPTIONS]
```
```
Options:
  -m, --mode <MODE>       Change the display mode of your device
  -s, --secondary <MODE>  Change the secondary display mode of your device (if supported)
      --pid <ID>          Specify the Product ID if you use mutiple devices
  -f, --fahrenheit        Change the temperature unit to °F
  -a, --alarm             Enable the alarm

Commands:
  -l, --list         Print Product ID of the connected devices
  -h, --help         Print help
  -v, --version      Print version
```

### Using Multiple Devices <sup>(optional)</sup>
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

# Automatic Start

## Systemd (Arch, Debian, Ubuntu, Fedora, etc.)
1. Copy the `deepcool-digital-linux` to the `/usr/sbin/` folder
```bash
sudo cp ./deepcool-digital-linux /usr/sbin/
```
2. Create the service file in the `/etc/systemd/system/` folder
```bash
sudo nano /etc/systemd/system/deepcool-digital.service
```
3. Insert the following:
```properties
[Unit]
Description=DeepCool Digital

[Service]
ExecStart=/usr/sbin/deepcool-digital-linux

[Install]
WantedBy=multi-user.target
```
4. Enable the service
```bash
sudo systemctl enable deepcool-digital
```
*Note: The program will run automatically after the next boot.*

## OpenRC (Gentoo, Artix Linux, etc.)
1. Copy the `deepcool-digital-linux` to the `/usr/sbin/` folder
```bash
sudo cp ./deepcool-digital-linux /usr/sbin/
```
2. Create the service file in the `/etc/init.d/` folder
```bash
sudo nano /etc/init.d/deepcool-digital
```
3. Insert the following:
```properties
#!/sbin/openrc-run

description="DeepCool Digital"
command="/usr/sbin/deepcool-digital-linux"
command_args=""
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

# Building from Source
For testing or customization, you can build the binary by following
the steps below.

## Dependencies
<details>
<summary><b>Arch-based distributions</b></summary>

1. Install the following packages
```bash
sudo pacman -S base-devel rustup
```
</details>

<details>
<summary><b>Debian-based distributions</b></summary>

1. Install the following packages
```bash
sudo apt install build-essential pkg-config libudev-dev curl
```
2. Install [rustup](https://rustup.rs/) (required to have the latest Rust compiler)
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
3. Update your current shell
```bash
. "$HOME/.cargo/env"
```
</details>

## Building
1. Clone the repository
```bash
git clone https://github.com/Nortank12/deepcool-digital-linux
```
2. Open the folder
```bash
cd deepcool-digital-linux
```
3. Run the build command
```bash
cargo build -r
```
You can find the binary inside the `./target/release` folder.

# More Information
[Device List and USB Mapping Tables](device-list/README.md)
