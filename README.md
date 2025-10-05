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
This CLI program is meant to replicate the functionality of the original `DeepCool Digital`
Windows program and I am gradually adding support to new devices.

If you have a device that has not been added or tested yet, please read the notes below the
supported devices.
If you think you can collaborate, please write an issue so we can get in touch.

# Installation
Simply download the latest [release](https://github.com/Nortank12/deepcool-digital-linux/releases)
and make it executable:
```bash
chmod +x deepcool-digital-linux
```
You will need root permission to send data to the device.

> [!TIP]
> For more accurate CPU temperature monitoring, you can use the [zenpower3](https://github.com/PutinVladimir/zenpower3)
> or [asus-ec-sensors](https://github.com/zeule/asus-ec-sensors) kernel modules on supported hardware.

> [!NOTE]
> On Intel's Arc GPUs, you have to use kernel version 6.13 or higher for proper temperature monitoring.

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


#### From Nixpkgs

`deepcool-digital-linux` is packaged in Nixpkgs, available under `pkgs.deepcool-digital-linux`. A
NixOS module is also available as `services.hardware.deepcool-digital-linux.enable`. In your NixOS
configuration (for example in your `configuration.nix`), enabling the service will add the package
to your `PATH` and start the Systemd service for the persistent daemon.

```nix
{
  # Enable deepcool-digital-linux
  services.hardware.deepcool-digital-linux.enable = true;

  # ... rest of your configuration ...
}
```


<!-- TODO: remove this once the Nixpkgs module receives an update -->

You may also want to add the relevant udev rules to your configuration if your hardware requires them.
In your configuration, you may use `services.udev.extraRules` to add any of the rules that you need.
This is an alternative to using paths such as `udev.d` that you might be used to from FHS distributions.

```nix
{
  # ... rest of our configuration ...
  services.udev.extraRules = ''
    # Intel RAPL energy usage file
    ACTION=="add", SUBSYSTEM=="powercap", KERNEL=="intel-rapl:0", RUN+="${pkgs.coreutils}/bin/chmod 444 /sys/class/powercap/intel-rapl/intel-rapl:0/energy_uj"

    # DeepCool HID raw devices
    SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3633", MODE="0666"

    # CH510 MESH DIGITAL
    SUBSYSTEM=="hidraw", ATTRS{idVendor}=="34d3", ATTRS{idProduct}=="1100", MODE="0666"
  '';
}
```

Once you enable the service, rebuild your configuration and reboot.

#### From Nix Flake

[this repository]: https://github.com/mzonski/deepcool-digital-linux/

You may wish to use the Nix flake provided by [this repository] to get an up-to-date build of
deepcool-digital-linux. While Nixpkgs might take some time to receive updates, the flake will
always remain up to date as it will build directly from source.

It is still possible to use the NixOS module provided by nixpkgs if using flake, but you
must adjust the modules' **package** option to use the correct package. In a setup with flakes
enabled, this would require you to pass `inputs` in `specialArgs`, and then obtain the package
from `inputs.deepcool-digital-linux.packages` as such:

```nix
{inputs, pkgs, ...}: {
  services.hardware.deepcool-digital-linux = {
    enable = true;
    package = inputs.deepcool-digital-linux.packages.${pkgs.system}.default;
  };
}
```

Do note that you will be building deepcool-digital-linux from the source each time the flake is
updated, because the Nixpkgs binary cache will not be able to provide you cached binaries.
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
        <td>AK500 DIGITAL PRO</td>
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
    <tr>
        <td>ASSASSIN IV VC VISION</td>
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
        <td align="center">✔️</td>
    </tr>
    <tr>
        <td>LP360</td>
        <td align="center">✔️</td>
    </tr>
    <tr>
        <td>LQ240</td>
        <td align="center">✅</td>
    </tr>
    <tr>
        <td>LQ360</td>
        <td align="center">✅</td>
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
        <td>CH270 DIGITAL</td>
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
        <td>CH690 DIGITAL</td>
        <td align="center">✔️</td>
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
      --pid <ID>          Specify the Product ID if multiple devices are connected
      --gpuid <VENDOR:ID> Specify the nth GPU of a specific vendor to monitor (use ID 0 for integrated GPU)

  -u, --update <MILLISEC> Change the update interval of the display [default: 1000]
  -f, --fahrenheit        Change the temperature unit to °F
  -a, --alarm             Enable the alarm
  -r, --rotate <DEGREE>   Rotate the display (LP Series only)

Commands:
  -l, --list         Print Product ID of the connected devices
  -g, --gpulist      Print all available GPUs
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
