# Trackrs <!-- omit in toc -->

Track your working time with Trackrs a simple Time Tracking CLI.

[![CI](https://github.com/ser-drephs/trackrs/actions/workflows/ci.yml/badge.svg)](https://github.com/ser-drephs/trackrs/actions/workflows/ci.yml)
[![CD](https://github.com/ser-drephs/trackrs/actions/workflows/cd.yml/badge.svg)](https://github.com/ser-drephs/trackrs/actions/workflows/cd.yml)

- [Usage](#usage)
    - [Start tracking](#start-tracking)
    - [Take a break](#take-a-break)
    - [End tracking](#end-tracking)
    - [Status](#status)
- [Configuration](#configuration)
- [Installation](#installation)
- [Build](#build)
    - [Using DevContainer](#using-devcontainer)
- [Contribution](#contribution)

## Usage

This section provides some usage examples.
For more information see the cli help `trackrs --help`.

### Start tracking

Execute `trackrs start` to add the initial entry.
After this the status is available.

### Take a break

Execute `trackrs break` to add a break entry.
Don't forget to contine after your break by executing `trackrs continue`.

### End tracking

Execute `trackrs end` to add an end entry and show the status for this day.

### Status

Execute `trackrs status` to get the current tracking status.

Status:
```
Work time:   03:20 (-05:30)
Online time: 03:10
Break:       00:10 (-00:20)

Started:     08:00
End:         16:30 (est.)
```

## Configuration

You can edit configuration by executing `trackrs config --edit`.

The config file is located at following location depending on the operating system.

Platform | Value | Example
--- | --- | ---
Linux | $HOME/.trackrs | /home/alice/.trackrs
macOS | $HOME/.trackrs | /Users/Alice/.trackrs
Windows | %USERPROFILE%/.trackrs | C:\Users\Alice\.trackrs

- `folder`: the folder for the time tracker json files.
- `threshold_limits`: time in minutes which acts as threshold for the limits. In between this limits neither `status` nor `end` will calculate additional breaks.
- `limits`: staring from a specific hour `start`, a mandatory `minutes` break is required. This is used by `status` and `end` to calculate the working time.
- `workperday`: setup the normal work time for a day in minutes.

Example:
```json
{
  "folder": "/root",
  "threshold_limits": 5,
  "limits": [
    {
      "start": 6,
      "minutes": 30
    },
    {
      "start": 8,
      "minutes": 45
    },
    {
      "start": 10,
      "minutes": 60
    }
  ],
  "workperday": {
    "monday": 480,
    "tuesday": 480,
    "wednesday": 480,
    "thursday": 480,
    "friday": 480,
    "saturday": 0,
    "sunday": 0
  }
}
```

## Installation

Current no automatic installation scripts or setups are available.
Download the binary from release and place it somewhere on your harddrive.
Will be added in the future maybe.

## Build

This project was developed with `rust`/`cargo` version 1.61.0.
Check [rust-lang.org](https://www.rust-lang.org/tools/install) on how to install rust.

Use `cargo install --path .` to get every up and running.

### Using DevContainer

Open this folder in VSCode. It should prompt to open a DevContainer and you are ready to go.

## Contribution

This repository uses semantic versioning and enforces conventional commits using Sailr.

Check [craicover/sailr](https://github.com/craicoverflow/sailr) for installation instuctions.
