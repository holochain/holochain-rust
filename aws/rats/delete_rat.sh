#!/usr/bin/env bash
#set -x

function spinner() {
    local info="$1"
    local pid=$!
    local delay=0.75
    local spinstr='|/-\'
    while kill -0 $pid 2> /dev/null; do
        local temp=${spinstr#?}
        printf " [%c]  $info" "$spinstr"
        local spinstr=$temp${spinstr%"$temp"}
        sleep $delay
        local reset="\b\b\b\b\b\b"
        for ((i=1; i<=$(echo $info | wc -c); i++)); do
            reset+="\b"
        done
        printf $reset
    done
    printf "    \b\b\b\b"
}

export AWS_DEFAULT_REGION=eu-central-1

stackStatus=$(aws cloudformation describe-stacks --stack-name "$1"-test-ecs-service --query 'Stacks[0].StackStatus' --output text)

if [[ $stackStatus == "CREATE_COMPLETE" ]]; then
    aws cloudformation delete-stack --stack-name "$1"-test-ecs-service

    stackStatus=$(aws cloudformation wait stack-delete-complete --stack-name "$1"-test-ecs-service) &
    spinner "Stack is being deleted, please wait"
    echo "///////////////"
    echo "Stack deleted"
else
    echo "$1-test-ecs-service does not exist"
fi