# Mina Local Network Setup with Docker Compose

This guide will help you set up a local Mina network with 1 seed node, 2 block producers, 1 snark coordinator and 1 snark worker.

## Prerequisites

- Ensure Docker is installed on your machine.

## Step-by-step Guide

1. **Create Required Directories**

Create necessary directories for your local Mina network:

```bash
mkdir -p ~/.minimina/default
```

2. **Generate Key Pairs for Block Producers**

We need to generate key pairs for our block producers `mina-bp-1` and `mina-bp-2` making sure that folder containing keys have appopriate file permissions. Keys for the seed node and snark coordinator will be hardcoded inside our docker-compose.

Use the following script to generate the necessary key pairs using the Docker image:

```bash
#!/bin/bash

declare -a bp_array=("mina-bp-1" "mina-bp-2")
declare bp_dir=~/.minimina/default/block_producer_keys
declare libp2p_dir=~/.minimina/default/libp2p_keys

# Create directories holding keys ensuring correct file permissions
mkdir -p $bp_dir $libp2p_dir
chmod 700 $bp_dir $libp2p_dir

for bp in "${bp_array[@]}"; do

    echo "----------------"
    echo "$bp keys: "
    echo
    
    # Generate block producer keys
    docker run \
    --rm \
    --env MINA_PRIVKEY_PASS='naughty blue worm' \
    --entrypoint mina \
    -v $bp_dir:/keys \
    gcr.io/o1labs-192920/mina-daemon:2.0.0rampup3-bfd1009-buster-berkeley \
    advanced generate-keypair -privkey-path /keys/$bp

    echo
    # Generate libp2p keys
    docker run \
    --rm \
    --env MINA_LIBP2P_PASS='naughty blue worm' \
    --entrypoint mina \
    -v $libp2p_dir:/keys \
    gcr.io/o1labs-192920/mina-daemon:2.0.0rampup3-bfd1009-buster-berkeley \
    libp2p generate-keypair -privkey-path /keys/$bp
    echo
done
```

3. **Generate Genesis Ledger File**

Generate genesis ledger file ensuring that generated keys for block producers will have funds to be able to produce blocks.

Here is a simple helper script generating `genesis_ledger.json` file in `~/.minimina/default` directory:

```bash
#!/bin/bash

# Define the path to the block producer keys
declare bp_dir=~/.minimina/default/block_producer_keys

# Read the contents of the public key files
declare mina_bp_1_key=$(<"$bp_dir/mina-bp-1.pub")
declare mina_bp_2_key=$(<"$bp_dir/mina-bp-2.pub")

# Write the JSON structure to ~/.minimina/default/genesis_ledger.json
declare genesis_ledger_path=~/.minimina/default/genesis_ledger.json
cat <<EOF > $genesis_ledger_path
{
  "genesis": {
    "genesis_state_timestamp": "2023-08-16T17:45:29+0200"
  },
  "ledger": {
    "name": "release",
    "num_accounts": 250,
    "accounts": [
     {
      "pk": "$mina_bp_1_key",
      "sk": null,
      "balance": "11550000.000000000",
      "delegate": null
     },
     {
      "pk": "$mina_bp_2_key",
      "sk": null,
      "balance": "11550000.000000000",
      "delegate": null
     }
    ]
  }
}
EOF

echo "Generated genesis ledger file in $genesis_ledger_path including keys:"
echo "Key 1: $mina_bp_1_key"
echo "Key 2: $mina_bp_2_key"
```

4. **Docker Compose Configuration**

Copy `docker-compose-example.yaml` to `~/.minimina/default/docker-compose.yaml`. 

```bash
cp docs/docker_compose_example/docker-compose-example.yaml ~/.minimina/default/docker-compose.yaml
```

5. **Start the Network**

Once everything is configured, spin up the local network.

```bash
cd ~/.minimina/default
docker compose up
```

And that's it! Your local Mina network should now be running. Monitor the logs to ensure all services are operating without errors.

> ⚠️ Depending on your Docker version, you might need to use `docker-compose up` and `docker-compose down` instead.

6. **Monitor and manage the network**

- To check running processes:

```bash
docker ps
```

- To view the logs of a specific Mina daemon (for example, mina-bp-1):

```bash
docker logs mina-bp-1 -f
```

- To check the status of a particular daemon (consult the `docker-compose.yaml` file to determine the client port for a specific daemon):

```bash
docker run \
--network host \
--rm \
--entrypoint mina \
gcr.io/o1labs-192920/mina-daemon:2.0.0rampup3-bfd1009-buster-berkeley \
client status -daemon-port 4000
```

- To stop and start particular daemon

```bash
docker stop mina-bp-2
docker start mina-bp-2
```

7. **Stop the network**

If you wish to stop the network, simply run:

```bash
cd ~/.minimina/default
docker compose down
```
