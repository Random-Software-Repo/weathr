# weathr
Shows U.S. National Weather Service forecasts on the command line!

Weathr queries the U.S. National Weather Service public API for current 
conditions and 7 day forecast for any locations serviced by the U.S. NWS.

The seven day forecast is presented in columns as will fit on your terminal:
```
Isla Vista, CA
        54°F
   This Afternoon         Tonight             Tuesday          Tuesday Night       New Year's Day     Wednesday Night
       Sunny           Partly Cloudy           Sunny            Mostly Clear           Sunny           Partly Cloudy
   High near 64°F      Low near 41°F       High near 63°F      Low near 46°F       High near 67°F      Low near 48°F
     5 mph ESE         5 to 10 mph NE         5 mph SE           5 mph WSW            5 mph NW           5 mph NNW
```

## Installing

```
make build
sudo make install
```

## Usage

### First use
```
weathr -l <decimal latitude>,<decimal longitude>
```


### Subsequent uses
```
weathr
```

weathr will cache the responses from the NWS as reload them as necessary.

