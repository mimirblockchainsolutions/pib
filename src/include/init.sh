#!/bin/bash

set -e

source pib.env

PROJECT_NAME=${PIB_PROJECT_NAME:?"Expecting PIB_PROJECT_NAME in pib.env"}

shopt -s nullglob # set `nullglob` to prevent iteration on empty match

for file in ./accounts/*.env
do
    ( # use subshell to sanitize env vars

    # extract variables from file
    source "$file"

    ACCOUNT_NAME=${PIB_ACCOUNT_NAME:?"Expecing PIB_ACCOUNT_NAME in $file"}
    ACCOUNT_PASS=${PIB_ACCOUNT_PASS:?"Expecting PIB_ACCOUNT_PASS in $file"}
    ACCOUNT_SECRET=${PIB_ACCOUNT_SECRET:?"Expecting PIB_ACCOUNT_SECRET in $file"}
    ACCOUNT_ADDR=${PIB_ACCOUNT_ADDR:?"Expecting PIB_ACCOUNT_ADDR in $file"}

    PASSWORD_FILE="${ACCOUNT_NAME}.pass"

    # set up password file
    echo $ACCOUNT_PASS > $PASSWORD_FILE

    # insert secret into store...
    ethstore insert $ACCOUNT_SECRET  $PASSWORD_FILE --dir keys/$PROJECT_NAME

    # ensure that secret decrypts..
    ethstore sign $ACCOUNT_ADDR $PASSWORD_FILE c82a3ca1f9436de9ffe54faf3fef7e7ac76897e02ba7fd5d013b840fd350d01b --dir keys/$PROJECT_NAME
    
    ) # end subshell
done

shopt -u nullglob # unset `nullglob` (not typically expected behavior


echo "OK"

