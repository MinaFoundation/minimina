#!/bin/bash

# Specify the folder path containing the account files
folder="resources/test-accounts/key-pairs"

# Create an array to store the account data
accounts=()

# Iterate over the files in the folder
for f in ./$folder/*.pub; do
    # Check if the file is a regular file

    # Extract the account number from the file name
    account_number=$(basename "$f" | cut -d "_" -f 2)

    # Extract the public key from the account file
    public_key=$(cat $f)

    # Add the account data to the array
    account_data="{\"pk\": \"$public_key\", \"sk\": \"\", \"balance\": \"155.000000000\", \"delegate\": null}"
    accounts+=("$account_data")
done

# Create the JSON structure
json="{\"accounts\": [$(IFS=,; echo "${accounts[*]}")]}"
echo "$json" > accounts-full.json
