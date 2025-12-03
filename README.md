[![License: AGPL v3](https://img.shields.io/badge/License-AGPL_v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)

# Fire Emblem Character Creator, 4th Edition (FECC4e)

A cross-platform desktop and web application for creating custom character portraits in the style of the Fire Emblem
series.

This is an update and full rewrite (in Rust) of the Fire Emblem Character Creator originally written in Java by
TheFlyingMinotaur, updated by BaconMaster120 and converted to Scarla by ValeTheVioletMote - these being considered the
1st, 2nd and 3rd 'versions' or 'editions' respectively. I use the word 'edition' instead, in the hope of myself
iterating with improvements and new features in future versions.

Many art assets are by Iscaneus.

## New Features of the 4th Edition

- **More Platform Support**: Runs natively on Windows, macOS, and Linux, and as a web application in browsers allowing
  use on any device with a web browser such as mobile devices.
- **Part Search** Conveniently find character parts you know the name of.
- **Drag and Drop** Intuitively drag assets to position them on the canvas.
- **Improved Asset Transformations** Rotation and resising use familiar mouse controls. Also you may now flip assets.

## Getting Started

### Web Application

The easiest way to use the Character Creator is by visiting the web application at <https://fecc.introverted.social>. No
installation/download required.

### Native Builds

Native builds for Windows, macOS, and Linux are available on
the [releases page](https://github.com/aidan-es/FECC4e/releases).

## Usage

The user interface is divided into three main sections: the Parts Panel on the left, the Colour Panel on the right, and
the Canvas in the centre.

1. **Selecting Parts**: Use the tabs in the Parts Panel to select different categories of character parts (e.g. Armour,
   Face, Hair). Click on a part's thumbnail to add it to the character. Clicking a selected part again will deselect it.
2. **Transforming Parts**: Once a part is on the canvas, click on it to select it. A bounding box with handles will
   appear:
    - **Move**: Click and drag the part to move it.
    - **Scale**: Click and drag the corner handles to scale the part.
    - **Rotate**: Click and drag the rotate handles to rotate the part.
3. **Colouring**: Use the Colour Panel to customise the colours for each aspect of the character. Each `Colourable` part
   has its own colour ramp, and you can either select from a predefined palette or choose a custom colour using the
   colour picker.
4. **Exporting**: Open the "Export" panel. From here, you can set the character's name, choose an output resolution, and save the portrait and/or token as a PNG
   image.
5. **Saving and Loading**: Open the "Save/Load" Panel You can save your character's configuration to a `.fecc` file, which can be loaded later to
   continue editing.

### Adding Your Own Art

To add your own custom assets, please refer to the [guide](https://fecc.introverted.social/art)).

## Building from Source (for Developers)

To build and run the application locally, you will need to have the Rust programming language toolchain installed.

1. **Clone the repository**

2. **This project uses `Trunk` for web builds. Install it with:**

    ```bash
    cargo install trunk
    ```

3  **Build and run:**
    -   **Native:**
        ```bash
        cargo run --release
        ```
    -   **Web:**
        ```bash
        trunk serve --open --release
        ```

### Contributing

Contributions are most welcome. Please feel free to open an issue, submit a pull request or contact me by email - hi@ the FECC domain listed above.

### Licence

**FECC4e** is open-source software licensed under the [GNU Affero General Public License v3.0 (AGPLv3)](LICENSE) -
excluding art assets.

Copyright (C) 2025 aidan-es.