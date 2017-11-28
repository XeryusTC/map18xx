18xx map and tile generator
===========================

Will build 18xx assets in a deterministic way. Currently focussed on tile
manifests, printable tile sheets and maps. The program takes a bunch of TOML
files which dictate what it should do. It outputs to SVG files, scaled to fit
A4 paper.

## Hexagon space
Items within a hex are usually given in hexagon-space. This is a 3D space
where the axis are at 60° to each other. An example of the axis is given below.
Note that the orientation of the axis when the hexagons are oriented with
horizontal edges differs from when the hexagons are oriented with vertical
edges.

Instead of using coordinates in hexagon-space there are these position codes
that can be used as a shortcut. North is the upper edge on a hexagon that has
horizontal edges, it is the top left edge on hexagons that are oriented
vertically.

* `N`:  north edge
* `NE`: north east edge
* `NW`: north west edge
* `S`:  south edge
* `SE`: south east edge
* `SW`: south west edge
* `C`:  center of hexagon

![Coordinate system](axes.svg)


## Modes
The program can operate in several different modes, they are described here.

### Definitions
By default the program runs in 'definitions' mode, meaning that it will output
all currently known tile definitions to a single file `definitions.svg`. The
output contains a list of tiles without a colour that can be used to define
other tiles in a game.

### Game mode
Generates a tile manifest and sheets with tiles to print and play and play a
specific game of 18xx.

# Command line arguments
A list of command line options is given below:

* `-h` and `--help`: display usage and help
* `-V` and `--version`: display version information and exit
* `-g` and `--game`: Requires a parameter MAP. Executes the program in game
	mode. Creates files for the game MAP defined in `games/`

# Tile definitions
To build a game you first need to know what tiles are available. To simplify
the specification of tiles there are tile definitions which cover possible
configurations of tiles. These definitions include the tracks and stations on
a tile, but they also dictate where revenue circles and other text on the tile
goes. The tile definitions live in `tiledefs/`, there is one tile definiton per
file. The filename (the part before the .toml) determines how the tile can be
refered to from other files. The contents of the file can include:

* An array of `path` tables, defining the lines on the tile.
* An array of `city` tables, defining the tokenable places on the tile.
* An array of `stop` tables, defining the non-tokanable revenue locations on
  the tile.
* A `is_lawson` parameter that makes the centre of a tile prettier.
* A code that allows further specifications to put large letters like `B` or
  `NY` on the tile.

The `is_lawson` parameter is a boolean that is `false` by default. It will draw
the centre of a tile neatly when multiple lines meet there. The tile number is
always drawn at the bottom right corner and has `text_id = 0`.

## `text_id` keys
Before discussing what can be defined on a tile it is necessary to know how to
specify text. To reduce the number of definitions required a consumer like a
tile manifest can determine what text goes on the tile. The tile definition can
specify where the text should go on the tile. To link these the consumer has
to supply an array of strings, the definition can then pick the string to use
from that array using the `text_id` (or similar) key. These IDs represent a
0-based index in the array of strings. The 0th element of the string array is
reserved for the tile number. It is recommended to keep the `text_id` field as
small as possible, otherwise the consumer will have to specify a lot of empty
strings. Multiple elements on a tile can use the same `text_id`, these will
all use the same string.

## `path` tables
To define the lines that run across a tile you can use an array of `path`
tables. Each `path` entry defines a single path. This usually looks like
```
[[path]]
start = "N"
end_pos = [0.1, 0.2, 0.3]
is_bridge = true
```

There are several things here. First are the `start` and `end` keys, these
define the start and end positions of a path respectively. These can take the
position codes that were defined in the 'Hexagon space' section.  Instead of
`start` and `end` the `start_pos` or `end_pos` can be used to specify a point
in hexagon-space. It is only allowed to use one of `start` and `start_pos`, the
same also holds for `end` and `end_pos`. It is however allowed to use `start`
and `end_pos` or vice versa together.

Usually paths are drawn with level crossings. If paths cross but it is not
allowed to switch there the `is_bridge` key can be used. Its default value is
`false`. When set to `true` it will cause white lines to be drawn along the
path it is specified on whenever it intersects with another path. It is only
necessary to specify it on one path when two paths cross.

## `city` tables
Cities which have a space for tokens can be defined using the `city` array of
tables. Each city defines a new set of up to 4 token circles with its own
revenue circle. It is currently not possible to rotate a city. A city table
can be defined as
```
[[city]]
circles = 2
position = "C"
text_id = 1
revenue_pos = [0.0, 0.6, 0.0]
```

The first key is the `circles` key, this determines how many token spots are
available. This can be any number between 1 and 4 inclusive, if another
amount is specified then a red circle will be drawn to indicate that it is an
invalid amount. The pair `position` and `pos` specify where to put the city,
the `position` key takes a position code. Usually it makes no sense to use
something else than `C` because the city would be drawn half off the tile. The
`pos` key is its complement, it allows you to define the position in
hexagon-space.

To define where the revenue should be located the `revenue_pos` key can be
used. This is always a hexagon-space coordinate. Along with `revenue_pos` there
is a `text_id` which specifies which string ends up as the revenue number. It
is suggested to set this to 1. If different cities earn different revenue they
should have different `text_id`

## `stop` tables
Small cities are always rendered as small black circles. In the future it may
become possible to render them as dashes as well. A stop is defined as
```
[[stop]]
position = [0.0, 0.0, 0.0]
text_id = 1
revenue_angle = 30
```

A stop must have these three fields, other fields are ignored. The `position`
key defines where a stop is positioned in hexagon-space. The `text_id` field
specifies which string is used as the revenue. The revenue box is always at
the same distance from a stop. You can specify where it goes with the
`revenue_angle` key, this is an angle in degrees at which the revenue circle
should be drawn relative to the stop.

## Tile `code`
Some tiles have a letter or code on them to restrict the upgrade path. Think
of `B` and `OO` tiles in 1830 for example. The string that is used for this
can be defined with `code_text_id` and its position is defined with the
`code_position` key. The `code_position` is always a coordinate in
hexagon-space.

# Game definition
Using the `-g`/`--game` command line parameter you can put the program into
game mode that will generate all files to pnp a game. The `-g`/`--game` flag
require an additional `MAP` parameter: the name of the game to generate files
for. This outputs a tile manifest in `manifest.svg` and a list of files called
`MAP-sheet-x.svg` where MAP is the name of the game and x is a sequence number.
Each of these sheet files can be directly printed on A4 paper. The manifest
file contains an example of each tile in the game together with a number that
indicates how many of those tiles are available during play.

## Tile manifest
A tile manifest consists of a list of tiles and a list of how often this tile
can be used in the game. The definition of a usable tiles is an array of `tile`
tables, it looks like
```
[[tile]]
color = "green"
base_tile = "52"
text = [
	"59",
	"40",
	"OO"
]
```

There are several elements here. The first is the `color` key, this defines
which color the tile is. Usual picks are yellow, green, russet and grey. The
next key is `base_tile` which specifies which tile definition is used. This
can be any TOML file in the `tiledefs/` directory. Finally is the `text`
array, it specifies the string that the `text_id`s in the tile definition
refer to. The first entry is always the displayed tile number, this does not
have to be the same as the `base_tile` key. The meaning of other entries
depends on what the tile definition used as `text_id`.

To specify the amounts available of each tile there is an `amounts` table, it
looks like
```
[amounts]
"1" = 1
"2" = 1
"3" = 2
"4" = 2
"7" = 4
```

It has a number of string keys, these are the tile number that were defined
in the first element of a tile's `text` array. After the equals sign is the
amount of tiles that are available for placement during the game.
