# Consensus simulations


## Usage

### Build

Run cargo build command under the general project folder (consensus-prototypes):

```shell
cargo build --profile release-opt --bin snow-family
```

Built binary is usually placed at `target/release-opt/consensus-simulations`,
or if built for a specific architecture (overridden) at `target/{ARCH}/release-opt/consensus-simulations`.

### Execute

Move binary at some place of your choice, or after build run (from the main project folder):

```shell
./target/release-opt/snow-family --help
```

```
consensus-simulations
Main simulation wrapper Pipes together the cli arguments with the execution

USAGE:
    snow-family.exe [OPTIONS] --input-settings <INPUT_SETTINGS> --output-file <OUTPUT_FILE>

OPTIONS:
    -f, --output-format <OUTPUT_FORMAT>      Output format selector [default: parquet]
    -h, --help                               Print help information
    -i, --input-settings <INPUT_SETTINGS>    Json file path, on `SimulationSettings` format
    -o, --output-file <OUTPUT_FILE>          Output file path

```

## SimulationSettings

Simulations are configured with a `json` settings description file like in the example:

```json
{
  "consensus_settings": {
    "snow_ball": {
      "quorum_size": 14,
      "sample_size": 20,
      "decision_threshold": 20
    }
  },
  "distribution": {
    "yes": 0.6,
    "no": 0.4,
    "none": 0.0
  },
  "byzantine_settings": {
    "total_size": 10000,
    "distribution": {
      "honest": 1.0,
      "infantile": 0.0,
      "random": 0.0,
      "omniscient": 0.0
    }
  },
  "wards": [
    {
      "time_to_finality": {
        "ttf_threshold" : 1
      }
    }
  ],
  "network_modifiers": [
    {
      "random_drop": {
        "drop_rate": 0.01
      }
    }
  ],
  "seed" : 18042022
}
```

### consensus_settings

`consensus_settings` is the consensus backend configuration, the following consensus are supported:

* [`snow_ball`](#Snowball)
* [`claro`](#Claro)

#### Snowball

Attributes:

* `quorum_size`: `usize`, `alpha` as per the snowball algorithm
* `sample_size`: `usize`, `K` as per the snowball algorithm
* `decision_threshold`: `usize`, `beta` as per the snowball algorithm

Example: 

```json
{
  "quorum_size": 14,
  "sample_size": 20,
  "decision_threshold": 20
}
```

#### Claro

Attributes:

* `evidence_alpha`: `f32`, `alpha` as per the claro algorithm
* `evidence_alpha_2`: `f32`, `alpha2` as per the claro algorithm
* `confidence_beta`: `f32`, `beta` as per the claro algorithm (AKA decision threshold)
* `look_ahead`: `usize`, `l` as per the claro algorithm
* `query`: `QueryConfiguration`:
  * `query_size`: `usize`, step node query size
  * `initial_query_size`: `usize`, base query size (usually same as `query_size`)
  * `query_multiplier`: `usize`, query size calculation in case no quorum found in step query
  * `max_multiplier`: `usize`, max query multiplier to apply

Example:

```json
{
  "evidence_alpha": 0.8,
  "evidence_alpha_2": 0.5,
  "confidence_beta": 0.8,
  "look_ahead": 20,
  "query": {
    "query_size": 30,
    "initial_query_size": 30,
    "query_multiplier": 2,
    "max_multiplier": 4
  }
}
```

### distribution

Initial **honest nodes** opinion distribution (**normalized**, must sum up to `1.0`)

* `yes`, initial yes distribution
* `no`, initial no distribution
* `none`, initial none opinion distribution 

Example: 

```json
{
  "yes": 0.6,
  "no": 0.4,
  "none": 0.0
}
``` 


### byzantine_settings

Byzantine nodes configuration

* `total_size`: `usize`, total number of nodes to be spawned
* `distribution`: **normalized** distribution on hones/byzantine nodes
  * `honest`: `f32`, **normalized** amount of hones nodes
  * `infantile`: `f32`, **normalized** amount of infantile nodes
  * `random`: `f32`, **normalized** amount of random nodes
  * `omniscient`: `f32`, **normalized** amount of omniscient nodes

Example:

```json
{
    "total_size": 10000,
    "distribution": {
      "honest": 1.0,
      "infantile": 0.0,
      "random": 0.0, 
      "omniscient": 0.0
    }
}
```

### Simulation style

Simulation can be done in different fashions:

* *Sync*, (**default**) nodes run per step at the same time, updating on the previous states.
* *Async*, nodes run per batches (*chunks*) of predefined sizes 
* *Glauber*, use the [glauber symulations solver](https://en.wikipedia.org/wiki/Glauber_dynamics)
  * `update_rate`, record network state every `update_rate` processed chunks.
  * `maximum_iterations`, threshold limit of simulation iterations

Example: 

```json
{
  ...,
  "simulation_style": "Sync"
}
```

```json
{
  ...,
  "simulation_style": {
    "Async" : {
      "chunks": 20
    }
  }
}
```

```json
{
  ...,
  "simulation_style": {
    "Glauber" : {
      "update_rate": 1000,
      "maximum_iterations":1000000
    }
  }
}
```

### wards

List of configurable experiment stop conditions based on the network state.

* `time_to_finality`, break when reaching a threshold number of consensus rounds
  * `ttf_threshold`: `usize`, threshold to be rebased



```json
[
    {
      "time_to_finality": {
        "ttf_threshold" : 1
      }
    }
]
```

* `stabilised`, break when for `n` rounds the network state keeps the same
  * `buffer`: `usize`, consecutive number of rounds or iterations to check
  * `check`: selector of what the ward should be aware of for checking states:
    * `rounds`: check by consecutive rounds
    * `iterations`: check every `n` iterations

```json
[
    {
      "stabilised": {
        "buffer" : 3,
        "check" : { "type": "rounds" }
      }
    }
]
```

or

```json
[
    {
      "stabilised": {
        "buffer" : 3,
        "check" : { "type": "iterations", "chunk":  100 }
      }
    }
]
```

* `converged`, break when a specified ratio of decided nodes is reached
  *  `ratio`, `[0.0-1.0]` range of decided nodes threshold

### network_modifiers

List of modifiers that handle the network state in each step iteration

* `random_drop`, drop a percentage of the votes (setting them up as `None`)
  * `drop_rate`: `f32`, normalize rate of dropped messages

Example:

```json
[
    {
      "random_drop": {
        "drop_rate": 0.01
      }
    }
]
```

### seed

The simulations can be run with a customized seed (otherwise is provided by the app itself) in order to make reproducible
runs. An `u64` integer must be provided

```json
{
  ...
  "seed" : 18042022
}
```

## Output format

Output is a [`Polars::Dataframe`](https://docs.rs/polars/latest/polars/frame/struct.DataFrame.html) [python version](https://pola-rs.github.io/polars/py-polars/html/reference/api/polars.DataFrame.html)

Columns are vote states for each round (from `0`, initial state, to experiment end round).

Three modes are supported, `["json", "csv", "parquet"]`, all of them standard dumps of `polars`.

### Votes

Votes are encoded as:
* `None` => `0`
* `Yes` => `1`
* `No` => `2`