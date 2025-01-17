# LP Series Mapping Table
<table>
    <tr>
        <th>DATA BYTE</th>
        <th>VALUE</th>
        <th>FUNCTION</th>
        <th>DISPLAY</th>
    </tr>
    <tr>
        <td>D0</td>
        <td>16</td>
        <td>REPORT ID</td>
        <td rowspan="6">-</td>
    </tr>
    <tr>
        <td>D1</td>
        <td>104</td>
        <td rowspan="5">FIXED HEADER</td>
    </tr>
    <tr>
        <td>D2</td>
        <td>1</td>
    </tr>
    <tr>
        <td>D3</td>
        <td>5</td>
    </tr>
    <tr>
        <td>D4</td>
        <td>29</td>
    </tr>
    <tr>
        <td>D5</td>
        <td>1</td>
    </tr>
    <tr>
        <td>D6</td>
        <td>0-247</td>
        <td>PIXELS - COLUMN 1</td>
        <td rowspan="14">ODD ROWS</td>
    </tr>
    <tr>
        <td>D7</td>
        <td>0-247</td>
        <td>PIXELS - COLUMN 2</td>
    </tr>
    <tr>
        <td>D8</td>
        <td>0-247</td>
        <td>PIXELS - COLUMN 3</td>
    </tr>
    <tr>
        <td>D9</td>
        <td>0-247</td>
        <td>PIXELS - COLUMN 4</td>
    </tr>
    <tr>
        <td>D10</td>
        <td>0-247</td>
        <td>PIXELS - COLUMN 5</td>
    </tr>
    <tr>
        <td>D11</td>
        <td>0-247</td>
        <td>PIXELS - COLUMN 6</td>
    </tr>
    <tr>
        <td>D12</td>
        <td>0-247</td>
        <td>PIXELS - COLUMN 7</td>
    </tr>
    <tr>
        <td>D13</td>
        <td>0-247</td>
        <td>PIXELS - COLUMN 8</td>
    </tr>
    <tr>
        <td>D14</td>
        <td>0-247</td>
        <td>PIXELS - COLUMN 9</td>
    </tr>
    <tr>
        <td>D15</td>
        <td>0-247</td>
        <td>PIXELS - COLUMN 10</td>
    </tr>
    <tr>
        <td>D16</td>
        <td>0-247</td>
        <td>PIXELS - COLUMN 11</td>
    </tr>
    <tr>
        <td>D17</td>
        <td>0-247</td>
        <td>PIXELS - COLUMN 12</td>
    </tr>
    <tr>
        <td>D18</td>
        <td>0-247</td>
        <td>PIXELS - COLUMN 13</td>
    </tr>
    <tr>
        <td>D19</td>
        <td>0-247</td>
        <td>PIXELS - COLUMN 14</td>
    </tr>
    <tr>
        <td>D20</td>
        <td>0-247</td>
        <td>PIXELS - COLUMN 14</td>
        <td rowspan="14">EVEN ROWS</td>
    </tr>
    <tr>
        <td>D21</td>
        <td>0-247</td>
        <td>PIXELS - COLUMN 13</td>
    </tr>
    <tr>
        <td>D22</td>
        <td>0-247</td>
        <td>PIXELS - COLUMN 12</td>
    </tr>
    <tr>
        <td>D23</td>
        <td>0-247</td>
        <td>PIXELS - COLUMN 11</td>
    </tr>
    <tr>
        <td>D24</td>
        <td>0-247</td>
        <td>PIXELS - COLUMN 10</td>
    </tr>
    <tr>
        <td>D25</td>
        <td>0-247</td>
        <td>PIXELS - COLUMN 9</td>
    </tr>
    <tr>
        <td>D26</td>
        <td>0-247</td>
        <td>PIXELS - COLUMN 8</td>
    </tr>
    <tr>
        <td>D27</td>
        <td>0-247</td>
        <td>PIXELS - COLUMN 7</td>
    </tr>
    <tr>
        <td>D28</td>
        <td>0-247</td>
        <td>PIXELS - COLUMN 6</td>
    </tr>
    <tr>
        <td>D29</td>
        <td>0-247</td>
        <td>PIXELS - COLUMN 5</td>
    </tr>
    <tr>
        <td>D30</td>
        <td>0-247</td>
        <td>PIXELS - COLUMN 4</td>
    </tr>
    <tr>
        <td>D31</td>
        <td>0-247</td>
        <td>PIXELS - COLUMN 3</td>
    </tr>
    <tr>
        <td>D32</td>
        <td>0-247</td>
        <td>PIXELS - COLUMN 2</td>
    </tr>
    <tr>
        <td>D33</td>
        <td>0-247</td>
        <td>PIXELS - COLUMN 1</td>
    </tr>
    <tr>
        <td>D34</td>
        <td>0-255</td>
        <td>D1-D33 CHECKSUM <sup><code>U8</code> REMAINDER</sup></td>
        <td rowspan="2">-</td>
    </tr>
    <tr>
        <td>D35</td>
        <td>22</td>
        <td>TERMINATION BYTE</td>
    </tr>
    <tr>
        <td>...</td>
        <td>...</td>
        <td>- NOT USED -</td>
        <td>...</td>
    </tr>
</table>

### PIXEL - BYTE Mapping
<table>
  <thead>
    <tr>
      <th>ROW NUM</th>
      <th>DEC</th>
      <th>HEX</th>
      <th>BIN</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <th>1 - 2</th>
      <td>16</td>
      <td>0x10</td>
      <td>0001 0000</td>
    </tr>
    <tr>
      <th>3 - 4</th>
      <td>32</td>
      <td>0x20</td>
      <td>0010 0000</td>
    </tr>
    <tr>
      <th>5 - 6</th>
      <td>64</td>
      <td>0x40</td>
      <td>0100 0000</td>
    </tr>
    <tr>
      <th>7 - 8</th>
      <td>128</td>
      <td>0x80</td>
      <td>1000 0000</td>
    </tr>
    <tr>
      <th>9 - 10</th>
      <td>1</td>
      <td>0x01</td>
      <td>0000 0001</td>
    </tr>
    <tr>
      <th>11 - 12</th>
      <td>2</td>
      <td>0x02</td>
      <td>0000 0010</td>
    </tr>
    <tr>
      <th>13 - 14</th>
      <td>4</td>
      <td>0x04</td>
      <td>0000 0100</td>
    </tr>
  </tbody>
</table>
