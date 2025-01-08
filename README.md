# weathr
Shows U.S. National Weather Service forecasts on the command line!

Weathr queries the U.S. National Weather Service public API for current 
conditions and 7 day forecast for any locations serviced by the U.S. NWS.

The seven day forecast is presented in columns as will fit on your terminal:
```
$ weathr
Waimanalo Beach, HI
Most recent observation from Kaneohe, Marine Corps Air Station(PHNG) at 11:57am: 79°F
       Today              Tonight             Thursday         Thursday Night          Friday           Friday Night
   Scattered Rain      Scattered Rain    Scattered Showers   Scattered Showers   Scattered Showers     Scattered Rain
      Showers             Showers        And Thunderstorms   And Thunderstorms   And Thunderstorms        Showers
   High near 79°F      Low near 73°F       High near 78°F      Low near 72°F       High near 79°F      Low near 73°F
   1 to 10 mph W       8 to 15 mph N      12 to 15 mph NE     13 to 16 mph ENE       18 mph ENE       17 to 21 mph ENE
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
For the output example above:
```
weathr -l 21.344,-157.703
```


### Subsequent uses
```
weathr
```

weathr will cache the responses from the NWS as reload them as necessary.

