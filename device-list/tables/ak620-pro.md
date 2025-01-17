# AK620 DIGITAL PRO Mapping Table
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
        <td>D1</td>
        <td>104</td>
        <td rowspan="7">FIXED HEADER</td>
    </tr>
    <tr>
        <td>D2</td>
        <td>1</td>
    </tr>
    <tr>
        <td>D3</td>
        <td>4</td>
    </tr>
    <tr>
        <td>D4</td>
        <td>13</td>
    </tr>
    <tr>
        <td>D5</td>
        <td>1</td>
    </tr>
    <tr>
        <td>D6</td>
        <td>2</td>
    </tr>
    <tr>
        <td>D7</td>
        <td>8</td>
    </tr>
    <tr>
        <td>D8</td>
        <td>0-255</td>
        <td rowspan="2">POWER CONSUMPTION <sup><code>U16</code></sup></td>
    </tr>
    <tr>
        <td>D9</td>
        <td>1-255</td>
    </tr>
    <tr>
        <td>D10</td>
        <td>0-1</td>
        <td>TEMPERATURE UNIT ˚C / ˚F</td>
    </tr>
    <tr>
        <td>D11</td>
        <td>0-255</td>
        <td rowspan="4">TEMPERATURE <sup><code>F32</code></sup></td>
    </tr>
    <tr>
        <td>D12</td>
        <td>0-255</td>
    </tr>
    <tr>
        <td>D13</td>
        <td>0-255</td>
    </tr>
    <tr>
        <td>D14</td>
        <td>1-255</td>
    </tr>
    <tr>
        <td>D15</td>
        <td>0-100</td>
        <td>UTILIZATION</td>
    </tr>
    <tr>
        <td>D16</td>
        <td>0-255</td>
        <td rowspan="2">FREQUENCY <sup><code>U16</code></sup></td>
    </tr>
    <tr>
        <td>D17</td>
        <td>1-255</td>
    </tr>
    <tr>
        <td>D18</td>
        <td>0-255</td>
        <td>D1-D17 CHECKSUM <sup><code>U8</code> REMAINDER</sup></td>
    </tr>
    <tr>
        <td>D19</td>
        <td>22</td>
        <td>TERMINATION BYTE</td>
    </tr>
    <tr>
        <td>...</td>
        <td>...</td>
        <td>- NOT USED -</td>
    </tr>
</table>
