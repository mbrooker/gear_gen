# gear_gen
G-Code generator for saw-cut spur gears, for use with involute gear cutters in plastic or metal, using a machine with a 4th axis. All cuts are conventional, no climb cuts (my machine isn't rigid enough for climb cutting with big tools in steel).

## Setup

### Machine Setup

- Rotary axis must be mounted as an A axis, on the left (negative X) side of the table.
- Pick the right cutter profile for the number of teeth, and mount it in the arbor.
- Measure the cutter diameter.
- Measure tool length to the center of the cutter, and make sure your machine is set up for tool length compensation on the tool you choose. Measuring accurately here is critical to getting a functional gear.

### Feeds and Speeds

Feeds and speeds depend on machine, material, and cutter (as always).

- 100m/min surface speed works will in Al with the "ebay" HSS cutters (~640 RPM)
- Feed around 60m/min works well with 500μm cuts

### Stock Setup
First, measure the stock to length, leaving at least one tool radius of stickout (but not much more). Then, cut to diameter (teeth + 2)\*modulus (see the Machinery's Handbook for tolerances for gears depending on the fit you want, but within 100μm (4 thou) is usually a good mix of easy to do and good enough for hobby work. Finally, mount the stock in the chuck of the rotary axis, and adjust the concentricity as necessary.

Probe the stock, putting the zero point in the middle of the right hand side of the stock.

Top-view (looking towards negative Z):

                                           +Y           
                                           ▲            
    ┌─────────────┐                        │            
    │             ├─┐                      │            
    │   Rotary    │ ├──────────────────────┤            
    │    Axis     │ │         Stock        ├──────▶  +X 
    │             │ ├──────────────────────┘            
    │             ├─┘                                   
    └─────────────┘                                     

Front view (looking towards negative X, along the A axis):

               +Z                
               ▲                 
               │                 
               │                 
    ┌──────────┼─────────┐       
    │      .───┼───.     │       
    │    ,'    │    `.   │       
    │  ,'      │      `. │       
    │ ;        │        :│       
    │ │       .┼.       ││       
    │ │      ( └────────┼┼───▶ +Y
    │ :       `─'       ;│       
    │  ╲               ╱ │       
    │   `.           ,'  │       
    │     `.       ,'    │       
    │       `─────'      │       
    └────────────────────┘
    
# The Cut

The cut proceeds right-to-left, behind (+Y) the stock. Solid lines are feed-rate moves, dashed lines are rapids:

                                 ▲ ─ ─ ─ ─ ─ ▶ │   
                                 │                 
                                 │             │   
                                 │                 
    ┌─────────────┐              │           .─▼─. 
    │             ├─┐            ◀──────────────  )
    │   Rotary    │ ├──────────────────────┐ `───' 
    │    Axis     │ │         Stock        │       
    │             │ ├──────────────────────┘       
    │             ├─┘                              
    └─────────────┘                                

The code will do multiple passes at different (-Y) depths. Adjust based on the capabilities of your machine, sharpness of your cutter, etc.
