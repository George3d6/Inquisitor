# ![Inquisitor icon](https://i.imgur.com/3XZNDko.png) Inquisitor
**_Version: 0.2.2_**


Inquisitor is a monitoring tool written solely in Rust, it's easy to extend via a plugin system, minimalist and easy to deploy,
fast and resource light by using asyncio and the amazing optimizations that rustc&llvm provide.

It's composed of a receptor, which receives messages, processes them, stores them and provides them to various endpoints,
and an agent, which is ran on the monitored machines in order to collect data via various plugins. Both the agent and the receptor have a similar plugin system,
the agent's plugins are meant for data collection and light processing, whilst the receptor plugins are meant for processing and correlating the data from the
various agents in order to generate more complex or generalized metrics.


## Usage

I'm currently working on a 20 minute video and a quick guide to explain how to deploy and use inquisitor (by showing how I use it to monitor my own infrastrucutre)


## Goals

### In short

- Written to be modified, extended and customized.

- Easy to deploy and run by providing an intuitive build, not having (many) dynamic dependencies and not consuming too many resources

- Effortless to learn, reading a 20 minute guide or watching a 40 minute video should be enough for anyone to understand how to use, deploy and extend this software.

- Meant for programatic use. The web-ui exists in order to provide some insights (In the future it may be distributed separately from the receptor), but the
intended use for the receptor is via extending it with your own custom endpoints that send warnings and reports.

### /Rant

The main reason why I thought this was worth developing is because most established monitoring tools (Zabbix, Nagios, Icinga, Munin) feel to "old" to me. Their
codebases are huge and impenetrable, the guides on how to use them could be compiled in a manual of hundreds of pages, they are relatively slow (sometimes by
their very design, due to the languages they are written in), they overxtend by adding security and authentication layers, they add even more bloat by trying to have interfaces aimed at "dumb/non-tech savy/business" users and often time are a mix between a community effort and an "enterprise" component

As such, the main reason why I want to develop Inquisitor is to have a monitoring tool that doesn't do that, something that is 100% free and open (no binary blobs,
no Enterprise Edition), something that is meant to be used by developer, that doesn't hold you hand and assumes your can't use a linux box, it assumes you are busy and want to deploy your monitoring software quickly, configure it quickly and have it work out of the box once you run the binary. Also, I want to build something that is
minimalist, that does one thing (monitoring) and does it well.


## Components

### The agent

The agent is ran on every monitored machine, so it's resource usage and binary size should be kept reasonably low.

The agent is the more complete part of the two. Containing most of the basic plugins one would need to engage in basic monitoring (resource monitoring, file
monitoring, process monitoring... etc). It's as dependency-free as I could make it without complicating myself, it's only dynamically linked only against
linux-vdso, libdl, librt, libpthread(to be removed when I figure out an easy way to force tokio to be single thread), ld and libc.

It's meant to send data to the receptor, it doesn't received data, thus not inducing the risk of a security breach if the receptor machine is compromised or the
receptor binary is corrupted or contains a bug.


### The receptor

The receptor is meant to run on only one or two instances and doesn't hamper the performance of other software, so most of the logic should be in the receptor if
at all possible.

The receptor is less fleshed out and, at the moment, it requires the sqlite3 shared library to be installed on the host machine. Besides receiving and processing
statuses from the agents, it also includes endpoints to query the data it has collected, a routines to process the received data via it's plugins and clean old data
as well as an endpoint for the inquisitor web_ui.


### Plugins

The plugin system is quite easy to get used to. It's biggest disadvantage is that each time a plugin has to be added or removed it requires a new compilation,
plugins can't be loaded dynamically or ran as seprate processes.


### The Web UI

This is the only part of the project that I'm not quite sure I want to keep. It's a minimalist UI that's meant to list and provide graphs for plugins. It isn't
able to automatically generate plots for new plugins, it requires a `timeseries_map_agent/recptor_Plugin_name` function to be defined in order to plot timeseries.
In the future, it will require a similar function for other types of graphs.

Unlike some other monitoring front-ends, it's not a top priority and it's not meant to be used for controlling the monitoring system. It's more of a showcase of
how you could build your own endpoints.


### Endpoints

Endpoints are meant to communicate the relevant data gathered by Inquisitor to the agent. Sending warning if process are down, if machine resources are
reaching beyond a certain cap or anthing else a user may desire.

Currently there are no example endpoints present (besides the web ui, which is a bit more special). In the next release some endpoints for slack, email and twillio will be included.



## Roadmap

#### <a href="#"><img alt="Feature" src="https://i.imgur.com/onvKoVz.png" height="28" width="28"></a> Feature / Change
#### <a href="#"><img alt="Bug" src="https://i.imgur.com/umZtkC4.png" height="28" width="28"></a> Bug
#### <a href="#"><img alt="Priority" src="https://i.imgur.com/6ieSrzD.png" height="28" width="28"></a> Priority
#### <a href="#"><img alt="Requires external contributors" src="https://i.imgur.com/lmOki5V.png" height="28" width="28"></a> Requires external contributors / Advice needed
#### <a href="#"><img alt="Under development" src="https://i.imgur.com/iSXfnTb.png" height="28" width="28"></a> Under development




<br>
<br>

<a href="#"><img alt="Feature" src="https://i.imgur.com/onvKoVz.png" height="28" width="28"></a>
<a href="#"><img alt="Priority" src="https://i.imgur.com/6ieSrzD.png" height="28" width="28"></a>
<a href="#"><img alt="Requires external contributors" src="https://i.imgur.com/lmOki5V.png" height="28" width="28"></a>
Come up with a testing plan in order to start moving towards a stable release. This part is a bit tricky since 99% of the "hard parts" relate to side effects,
so it may require a bit of fiddling to come up with a good testing framework and practices. (Advice is welcome on this one)


<a href="#"><img alt="Feature" src="https://i.imgur.com/onvKoVz.png" height="28" width="28"></a>
<a href="#"><img alt="Under development" src="https://i.imgur.com/iSXfnTb.png" height="28" width="28"></a>
<a href="#"><img alt="Requires external contributors" src="https://i.imgur.com/lmOki5V.png" height="28" width="28"></a>
Finalize the http interface for getting plugin data and make sure it's not missing any essential component before starting to build towards a stable release.



<a href="#"><img alt="Feature" src="https://i.imgur.com/onvKoVz.png" height="28" width="28"></a>
<a href="#"><img alt="Priority" src="https://i.imgur.com/6ieSrzD.png" height="28" width="28"></a>
Replace the "custom" code inlining system with some macros (if possible).



<a href="#"><img alt="Bug" src="https://i.imgur.com/umZtkC4.png" height="28" width="28"></a>
<a href="#"><img alt="Priority" src="https://i.imgur.com/6ieSrzD.png" height="28" width="28"></a>
Make sure SQLite starts in serialized mode (and compiles in serialized mode once SQLite compilation and static linking is added)


<a href="#"><img alt="Feature" src="https://i.imgur.com/onvKoVz.png" height="28" width="28"></a>
Add more pretty graphs to the web ui, add a more intuitive way for people to add plugins to the web ui.


<a href="#"><img alt="Feature" src="https://i.imgur.com/onvKoVz.png" height="28" width="28"></a>
Decide whether or not to separate the Web UI from the receptor component (at the moment it seems like it may fare better as a standalone endpoint)

<a href="#"><img alt="Feature" src="https://i.imgur.com/onvKoVz.png" height="28" width="28"></a>
Make the Web UI mobile friendly


<a href="#"><img alt="Feature" src="https://i.imgur.com/onvKoVz.png" height="28" width="28"></a>
Add SQLite compilation as part of the build and link with it statically


<a href="#"><img alt="Feature" src="https://i.imgur.com/onvKoVz.png" height="28" width="28"></a>
Add some more plugin to the receptor side of things, such as average resource usage plugin, text diff check (to, for example, check the differences between running
    the same command on multiple machine)... etc
