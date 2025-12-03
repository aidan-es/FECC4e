---
description: A guide on how to create and add custom art assets to the FE Character Creator.
---
# FECC4e Art

## How to Add Art Files

- If you're using a FECC4e native application, simply place the file into the 'art' directory that you'll find in the
  archive folder next to the executable.
- If you're using a FECC4e web application:
    - Click 'Add Art' on the menu bar.
    - Click 'Upload File'.
    - Select the desired file from your OS file picker.

  Note: art added to the webapp does not persist currently - if you close or refresh the page and wish to use it in a
  new design, you will need to add it again.

## How to Make FECC4e Compatible Art Files.

Token files must be 64x64 pixels and end with "_Token.png". All other types must be 96x96 pixels and end with '_
Face.png', '_Hair.png', '_HairBack.png', '_Armour.png' or '_Accessory.png' respectively.

The colouring system is rather esoteric, but must be followed for compatibility with all the existing artwork. RGB
colours consist of a red, green and blue channel, only the red channel is considered by FEEC4e when reading art files.
The values you set for green and blue are imaterial.

Differing red values are used to differeniate different parts of the art that should be coloured differently. Red values
are divided by 10 and then compared to the bellow keys (so multiply these key values by 10 to get the red values you
should use in your art).

### For Face and Accessory files:

0 - Outline Colour

1 - Eye and Beard Colour (light)

2 - Eye and Beard Colour (base)

3 - Eye and Beard Colour (dark)

4 - Skin Colour (light)

5 - Skin Colour (base)

6 - Skin Colour (dark)

7 - Skin Colour (darker)

8 - Skin Colour (darkest)

9 - Accessory Colour (light)

10 - Accessory Colour (base)

11 - Accessory Colour (dark)

### For Armour and Hair files:

0 - Outline Colour

1 - Hair Colour (light)

2 - Hair Colour (base)

3 - Hair Colour (dark)

4 - Skin Colour (light)

5 - Skin Colour (base)

6 - Skin Colour (dark)

7 - Skin Colour (darker)

8 - Skin Colour (darkest)

9 - Metal Colour (light)

10 - Metal Colour (base)

11 - Metal Colour (dark)

12 - Trim Colour (light)

13 - Trim Colour (base)

14 - Trim Colour (dark)

15 - Cloth Colour (light)

16 - Cloth Colour (base)

17 - Cloth Colour (dark)

18 - Leather Colour (light)

19 - Leather Colour (base)

20 - Leather Colour (dark)


