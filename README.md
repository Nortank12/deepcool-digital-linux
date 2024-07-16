# About
This program is meant to replicate the functionality of the original `DeepCool Digital`
Windows program and I am gradually adding support to new devices.

If you have a device that has not been added or tested yet, please read the notes below the
supported devices.
If you think you can collaborate, please write an issue so we can get in touch.

# Installation
Simply download the latest [release](https://github.com/Nortank12/deepcool-digital-linux/releases)
and run it in the command line. You will need root permission to send data to the device.

I built the binary for `x86_64` architecture and tested it on Arch, Debian, and Gentoo but it should
work on any other Linux distribution.

## Supported Devices
<table>
    <tr>
        <th>PID</th>
        <th>Model</th>
        <th>Added</th>
        <th>Tested</th>
    </tr>
    <tr>
        <td>1</td>
        <td>AK400 DIGITAL</td>
        <td align="center">✅</td>
        <td align="center"></td>
    </tr>
    <tr>
        <td>2</td>
        <td>AK620 DIGITAL</td>
        <td align="center">✅</td>
        <td align="center"></td>
    </tr>
    <tr>
        <td>3</td>
        <td>AK500 DIGITAL</td>
        <td align="center">✅</td>
        <td align="center"></td>
    </tr>
    <tr>
        <td>4</td>
        <td>AK500S DIGITAL</td>
        <td align="center">✅</td>
        <td align="center">✅</td>
    </tr>
    <tr>
        <td>5</td>
        <td>CH560 DIGITAL</td>
        <td align="center"></td>
        <td align="center"></td>
    </tr>
    <tr>
        <td>6</td>
        <td>LS720 SE DIGITAL</td>
        <td align="center"></td>
        <td align="center"></td>
    </tr>
    <tr>
        <td>7</td>
        <td>??</td>
        <td align="center"></td>
        <td align="center"></td>
    </tr>
    <tr>
        <td>8</td>
        <td>AG400 DIGITAL</td>
        <td align="center"></td>
        <td align="center"></td>
    </tr>
    <tr>
        <td>9</td>
        <td>??</td>
        <td align="center"></td>
        <td align="center"></td>
    </tr>
    <tr>
        <td>10</td>
        <td>LD240</td>
        <td align="center">✅</td>
        <td align="center">✅</td>
    </tr>
    <tr>
        <td>...</td>
        <td>...</td>
        <td align="center"></td>
        <td align="center"></td>
    </tr>
    <tr>
        <td>21</td>
        <td>CH360 DIGITAL</td>
        <td align="center"></td>
        <td align="center"></td>
    </tr>
</table>

- *If your device is not on the list, you can still run the program and see if it detects it.*

- *If your device is on the list, please try to check all the features if they work as expected.*

*In any case, you can create an issue or add a comment to an existing one.*

# Usage
You can run the program with or without providing any options.
```bash
sudo ./deepcool-digital-linux [OPTIONS]
```
```bash
Options:
  -m, --mode <MODE>  Change the display mode between "temp, usage, auto" [default: temp]
  -f, --fahrenheit   Change temperature unit to Fahrenheit
  -a, --alarm        Enable the alarm (85˚C | 185˚F)
  -h, --help         Print help
  -V, --version      Print version

```

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

## OpenRC (Gentoo)
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

# Debug Information
<details>
    <summary>AK Series</summary>
    <table>
        <tr>
            <th>DATA BYTE</th>
            <th>VALUE</th>
            <th>FUNCTION</th>
        </tr>
        <tr>
            <td>D0</td>
            <td>16</td>
            <td>REPORT ID</td>
        </tr>
        <tr>
            <td rowspan="4">D1</td>
            <td>170</td>
            <td>STATUS BAR ANIMATION</td>
        </tr>
        <tr>
            <td>19</td>
            <td>TEMPERATURE MODE ˚C</td>
        </tr>
        <tr>
            <td>35</td>
            <td>TEMPERATURE MODE ˚F</td>
        </tr>
        <tr>
            <td>76</td>
            <td>USAGE MODE</td>
        </tr>
        <tr>
            <td>D2</td>
            <td>1-10</td>
            <td>STATUS BAR VALUE</td>
        </tr>
        <tr>
            <td rowspan="2">D3</td>
            <td>141</td>
            <td>MINUS SIGN</td>
        </tr>
        <tr>
            <td>1-9</td>
            <td>NUMERIC DISPLAY || Oxx</td>
        </tr>
        <tr>
            <td>D4</td>
            <td>1-9</td>
            <td>NUMERIC DISPLAY || xOx</td>
        </tr>
        <tr>
            <td>D5</td>
            <td>1-9</td>
            <td>NUMERIC DISPLAY || xxO</td>
        </tr>
        <tr>
            <td>D6</td>
            <td>1</td>
            <td>ALARM ON</td>
        </tr>
            <tr>
            <td>...</td>
            <td>...</td>
            <td>- NOT USED -</td>
        </tr>
    </table>
</details>

# Development
### LD Series: [asdfzdfj](https://github.com/asdfzdfj) / [deepcool-ld-digital-hidapi](https://github.com/asdfzdfj/deepcool-ld-digital-hidapi)
