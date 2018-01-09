18xx map and tile generator
===========================

Will build 18xx assets in a deterministic way. Currently focussed on tile
manifests, printable tile sheets and maps. The program takes a bunch of JSON
files which dictate what it should do. It outputs to SVG files, scaled to fit
A4 paper.

## Hexagon space
Items within a hex are usually given in hexagon-space. This is a 3 dimensional
space where the axis are at 60° to each other. An example of the axis is given
below. Note that the orientation of the axis when the hexagons are oriented
with horizontal edges differs from when the hexagons are oriented with vertical
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

When specifying coordinates a position code can be used by inserting
`{"Named": "<code>"}` in the appropriate location. `<code>` is one of the
above position codes. When specifying a coordinate in hexagon-space it done
done by setting the coordinate to `{"HexSpace": [<x>, <y>, <z>]}` where `<x>`,
`<y>` and `<z>` are floating point numbers. It is important to always include
at least one decimal even when not required (so 10.0 instead of 10). Otherwise
map18xx will not be able to understand it.

## Modes
The program can operate in several different modes, they are described here.

### Definitions
By default the program runs in 'definitions' mode, meaning that it will output
all currently known tile definitions to a single file `definitions.svg`. The
output contains a list of tiles without a colour that can be used to define
other tiles in a game.

### Game mode
Generates a tile manifest (lists each tile in the game and how many there are
in total) and sheets with tiles to print and play and play a specific game of
18xx. It will also generate a map on which the tiles can be placed.

# Command line arguments
A list of command line options is given below:

* `-h` and `--help`: display usage and help
* `-V` and `--version`: display version information and exit

# Tile definitions
To build a game you first need to know what tiles are available. To simplify
the specification of tiles there are tile definitions which cover possible
configurations of tiles. These definitions include the tracks and stations on
a tile, but they also dictate where revenue circles and other text on the tile
goes. The tile definitions live in `tiledefs/`, there is one tile definiton per
file. The filename (the part before the .json) determines how the tile can be
refered to from other files. The contents of the file can include:

* An array called `paths`, defining the lines on the tile.
* An array called `cities`, defining the tokenable places on the tile.
* An array called `stops`, defining the non-tokenable revenue locations on the
  tile.
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
reserved for the tile number. It is recommended to keep the id of the `text_id`
field as small as possible, otherwise the consumer will have to specify a lot
of empty strings. Multiple elements on a tile can use the same `text_id`, these
will all use the same string.

## `paths` array
To define the lines that run across a tile you can use the `paths` array. Each
entry defines a single path. They usually look like
```JSON
{
	"start": { "Named": "N" },
	"end": { "HexSpace": [0.1, 0.2, 0.3] },
	"is_bridge": true
}
```

There are several things here. First are the `start` and `end` keys, these
define the start and end positions of a path respectively. These can take the
position codes and hexagon-space coordinates that were defined in the
'Hexagon space' section.

Usually paths are drawn with level crossings. If paths cross but it is not
allowed to switch there the `is_bridge` key can be used. Its default value is
`false`. When set to `true` it will cause white lines to be drawn along the
path it is specified on whenever it intersects with another path. It is only
necessary to specify it on one of the crossing paths.

## `cities` array
Cities which have a space for tokens can be defined using the `cities` array.
Each city defines a new set of up to 4 token circles with its own revenue
circle. It is currently not possible to rotate a city. A city can be defined as
```JSON
{
	"circles": 2,
	"position": { "Named": "C" },
	"text_id": 1,
	"revenue_position": { "HexSpace": [0.0, 0.6, 0.0]
}
```

The first key is the `circles` key, this determines how many token spots are
available. This can be any number between 1 and 4 inclusive, if another
amount is specified then a red circle will be drawn to indicate that it is an
invalid amount. The `position` key specifies where to put the city. Usually it
doesn't make sense to use a position code other than `{ "Named": "C" }`
because the city would be drawn half off the tile.

To define where the revenue should be located the `revenue_position` key can be
used. It is recommended to use a hexagon-space coordinate. Along with
`revenue_position` there is a `text_id` which specifies which string ends up as
the revenue number. It is suggested to set this to 1 for consistency. If
different cities earn different revenue they should have a different `text_id`.

## `stops` array
Small cities are always rendered as small black circles. In the future it may
become possible to render them as dashes as well. A stop is defined as
```JSON
{
	"position": { "HexSpace": [0.0, 0.0, 0.0] },
	"text_id": 1,
	"revenue_angle": 30
}
```

A stop must have these three fields, other fields are ignored. The `position`
key defines where a stop is positioned. The `text_id` field specifies which
string is used as the revenue. The revenue circle is always at the same
distance from a stop. You can specify where it goes with the `revenue_angle`
key. This is the angle in degrees at which the revenue circle should be drawn
relative to the stop.

## Tile `code`
Some tiles have a letter or code on them to restrict the upgrade path. Think
of `B` and `OO` tiles in 1830 for example. The string that is used for this
can be defined with `code_text_id` and its position is defined with the
`code_position` key. The `code_position` can be a position code or a coordinate
in hexagon-space.

# Game definition
By using `game` for the mode option mode you can put the program into game mode
that will generate all files to PnP a game. The game mode requires an
additional `NAME` parameter; this is the name of the game to generate files
for. The available games are the sub directories in the `games` directory. Thisoutputs a tile manifest in `manifest.svg` and a list of files called
`NAME-sheet-x.svg` where NAME is the name of the game and x is a sequence
number. Each of these sheet files can be directly printed on A4 paper. The
manifest file contains an example of each tile in the game together with a
number that indicates how many of those tiles are available during play.

## Tile manifest
A tile manifest consists of a list of tiles and a list of how often this tile
can be used in the game. The definition of a usable tiles is given by the
`tiles` array. A single entry looks like
```JSON
[
	"color": "green",
	"base_tile": "52",
	"text": [
		"59",
		"40",
		"OO"
	]
]
```

There are several elements here. The first is the `color` key, this defines
which color the tile is. Usual picks are yellow, green, russet and grey. The
next key is `base_tile` which specifies which tile definition is used. This
can be any JSON file in the `tiledefs/` directory. Finally is the `text`
array, it specifies the string that the `text_id`s in the tile definition
refer to. The first entry is always the displayed tile number, this does not
have to be the same as the `base_tile` key. The meaning of other entries
depends on what the tile definition used as `text_id`.

To specify the amounts available of each tile there is an `amounts` array, it
looks like
```JSON
"amounts": {
	"1": 1,
	"2": 1,
	"3": 2,
	"4": 2,
	"7": 4
}
```

It has a number of string keys, these are the tile number that were defined
in the first element of a tile's `text` array. After the colon is the amount of
tiles that are available for placement during the game.
