# About
I have created a small program to replicate the functionality of the original `Deepcol Digital`
Windows program.

Currently, I have tested it with my AK series cooler and the rest of the devices
should be compatible. I plan to add support for cases as well, but I am lacking information.

If you have a different device and can provide some information, please write me an issue.
If you think you could collaborate, please write an issue so we can get in touch.

# Installation
Simply download the latest [release](https://github.com/Nortank12/deepcool-digital-linux/releases)
and run it in the command line. You will need root permission to send data to the device.

I built the binary for `x86_64` architecture and tested on Debian. If you encounter any issues,
let me know.

## Supported Devices
<table>
    <tr>
        <th>Model</th>
        <th>Added</th>
        <th>Tested</th>
    </tr>
    <tr>
        <td>AK400 DIGITAL</td>
        <td align="center">✅</td>
        <td align="center"></td>
    </tr>
    <tr>
        <td>AK620 DIGITAL</td>
        <td align="center">✅</td>
        <td align="center"></td>
    </tr>
    <tr>
        <td>AK500 DIGITAL</td>
        <td align="center">✅</td>
        <td align="center"></td>
    </tr>
    <tr>
        <td>AK500S DIGITAL</td>
        <td align="center">✅</td>
        <td align="center">✅</td>
    </tr>
</table>

# Usage
You can run the program with or without providing any options.
```bash
sudo ./deepcool-digital-linux [OPTIONS]
```
```bash
Options:
  -m, --mode <MODE>  Change the display mode between "temp, usage, auto" [default: temp]
  -f, --fahrenheit   Change temperature unit to Fahrenheit
  -a, --alarm        Enable the alarm (80˚C or 176˚F)
  -h, --help         Print help
  -V, --version      Print version

```

# Automatic start
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
Description=Deepcool Digital

[Service]
ExecStart=/usr/sbin/deepcool-digital-linux # arguments here

[Install]
WantedBy=multi-user.target
```
4. Enable the service
```bash
sudo systemctl enable deepcool-digital
```

# Debug Information
If you want to test your device, this chart shows how the raw data looks like for the AK
series displays.
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
