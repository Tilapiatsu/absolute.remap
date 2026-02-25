# absolute.remap
## Description
Simple comandline tool to remap tablet stylus inputs with evdev written in rust

It creates a evdev virtual device to ovrride the default behaviour of the stylus in the following way :

```
BTN_STYLUS -> null
BTN_STYLUS2 -> null
BTN_SYLUS + BTN_TOUCH -> BTN_RIGHT
BTN_SYLUS2 + BTN_TOUCH -> BTN_MIDDLE
```

All other pen absolute InputEvents are passed without changes like ABS_X, ABS_Y, ABS_PRESSURE etc ...

## Parameters

Run command with :
```
absolute-remap --help
```