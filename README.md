# literate-sniffle
Monitoring websites for changes using Rust and Git

## Use:
This command-line tool takes one of the following arguments:
- `setup (<file>)`: generates configuration file as ~/.sniffer-config, sets up tracking directory and log files in user specified location. If `<file>` is specified, initializes using a pre-existing list of resources to track.
- `add <name> <url>`: initialize tracking repo for `<url>`, with alias `<name>`
- `update <name>`: update tracking repo for `<name>`
- `all`: update all currently tracked urls
- `diffs <name>`: get changes between two most recent versions of `<name>`
- `names`: list currently assigned names and associated urls

### Configuration file format:
To set up using an existing list of resources, the file should be in the following format:  
`name-1`  `url-1`  
`name-2`  `url-2`  
...

## To-do
- Currently only really handles resources that can be interpreted as text. Should take into account data type when dealing with resources.
- Allow tracking of multiple resources under the same domain?
