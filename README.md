# ![Inquisitor icon](https://i.imgur.com/3XZNDko.png) Inquisitor
**_Version: 0.3.1_**

Inquisitor is a monitoring tool written solely in Rust, it's easy to extend via a plugin system, minimalist and easy to deploy,
fast and resource light by using asyncio and the amazing optimizations that rustc&llvm provide.

It's composed of a receptor, which receives messages, processes them, stores them and provides them to various endpoints,
and an agent, which is ran on the monitored machines in order to collect data via various plugins. Both the agent and the receptor have a similar plugin system, the agent's plugins are meant for data collection and light processing, whilst the receptor plugins are meant for processing and correlating the data from the various agents in order to generate more complex or generalized metrics.


## Usage

### Video guides

[![Inquisitor setup tutorial (version 0.2.2)](http://www.clipartbest.com/cliparts/9cp/onE/9cponEedi.png)](https://www.youtube.com/watch?v=Nbn-85oAcRU&feature=youtu.be)

### Text guides

#### Receptor setup

The receptor is the component that receives statuses from the various machine being monitored. It can also process them in order to provide
higher level metrics. It serve these endpoints over a http interface (by default on port 1834). To install and run the receptor:

1. Go to the machine where the receptor is to be installed and make sure these are netowrking rules in places such that it can receive
messages from the monitored machines on port `1478` and serve those statuses on port `1834`

2. Download the latest release of the receptor from https://github.com/George3d6/Inquisitor/releases and uncompress it.
`wget 'https://github.com/George3d6/Inquisitor/releases/download/0.2.2/inquisitor-receptor.tar.gz' && tar -xvf inquisitor-receptor.tar.gz && rm inquisitor-receptor.tar.gz;`

4. `cd inquisitor-receptor` and edit the `receptor_config.yml` file to allow it to bind to a public interface (unless you are testing locally).
You can do this by changing the `bind` parameter to `0:0:0:0`

5. Run the binary `./inquisitor_receptor`

6. You should be able to get data from the following routes:

* The web UI: `your_ip:1834`

* The list plugin route: `your_ip:1834/plugin_list?level=agent` Where `level` is either agent (for agent plugins) or receptor (for receptor plugins)

* The plugin data route: `your_ip:1834/plugin_data?ts_start=0&ts_end=1920197300&level=agent&name=System%20monitor`
Where `ts_start` is the timestamp of the oldest status you want to get and `ts_end` the date of the newest. `plugin_name` is the name of the
plugin (You can list all plugin you have data fro from the List plugin route).

#### Agent setup

1. On the machines you want to monitor, download the latest release of the agent from https://github.com/George3d6/Inquisitor/releases and uncompress it.
`wget 'https://github.com/George3d6/Inquisitor/releases/download/0.2.2/inquisitor-agent.tar.gz' && tar -xvf inquisitor-agent.tar.gz && rm inquisitor-agent.tar.gz;`

2. In the `agent_config.yml` file set `host` to the IP/Domain of the machine were you installed the receptor. Add a `machine_identifier`
parameter if you want the identifier/name for this machine to be something else than it's hostname

3. Edit the `plugin_name.yml` files for any plugin you want to use (e.g. `file_checker.yml`). To enable the plugin set `enabled: true`,
all plugins are disabled by default.

4. Run the agent `./inquisitor_agent`. It should start sending data to the receptor. You can check the data being send at the endpoints
listed above.


#### Endpoint setup

##### Web endpoint


##### Slack endpoint


#### Plugin configuration guide

@TODO


## Goals

- A monitoring tool meant to be modified, extended and customized.

- Easy to deploy and run by providing an intuitive build, not having (many) dynamic dependencies and not consuming too many resources

- Effortless to learn, 15 minute should be enough time to learn how to use this software almost perfectly. An hour should be enough to learn
how to add your own custom plugins to it.

- Meant for programatic use. The interface should be built in such a way as to easily allow developers/devops to add their own custom endpoints.


## Components

### The agent

Meant to be installed on all the machine you want to monitor. It's a scaffolding that allows plugins to send data to the receptor.


### The receptor

Collects, processes and stored data from all the agents in order to display it. The processing of data can be modified via plugins.


### Plugins

Plugins can be added to both the receptor and the agent in order to gather different types of data and, if needed, process it in different
ways in the receptor. Plugins are meant to be written in rust and should be easy to configure and must implement the `AgentPlugin`
or `ReceptorPlugin` trait. For bare-bones plugin examples see [the Alive plugin (for the agent)](agent_plugins/alive.rs) and
[the Sync check plugin (for the receptor)](receptor_plugins/sync_check.rs)

### The Web UI

A minimalist UI that's meant to list plugins and provide graphs for them. It isn't
able to automatically generate plots for a new plugins, it requires a `timeseries_map_agent/recptor_Plugin_name` function to be defined in order to plot timeseries. In the future, it will require a similar function for other types of graphs.

In the future it may be made into a standalone endpoint, for now it is part of the inquisitor-receptor server.


### Endpoints

Endpoints are meant to communicate the relevant data gathered by Inquisitor to the agent. Sending warning or periodic status messages depending
on the statuses received from the agents or the data processed by the receptor.

Currently the only endpoint present is the web ui, I'm working on implementing some example endpoints that can send warnings via twillio,
slack and SMTP.


## Extending via endpoints and plugins & contributing code
@TODO
