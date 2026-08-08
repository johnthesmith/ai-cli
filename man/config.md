# Configuration

1. Configuration of ai-cli can change the following:
    1. [Config file](#config-file)
    2. [cli](#cli)



## Config file

1. The config file contains the main settings.
2. The file will be created after `--init` command in the
`./.ai-cli/profiles/<profile>/comfig.yaml` file.



## cli

1. You can override root config arguments from the CLI, using the
notation `--key=value`.



# For developers

1. The config template is embedded into the source code
[config_template.rs](/src/ai/config_template.rs).
