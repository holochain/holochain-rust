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

#export AWS_DEFAULT_REGION=us-east-1
#export AWS_DEFAULT_REGION=ap-southeast-2
export AWS_DEFAULT_REGION=eu-central-1

stackStatus=$(aws cloudformation describe-stacks --stack-name "$1"-test-ecs-service --query 'Stacks[0].StackStatus' --output text)

if [[ $stackStatus == "CREATE_COMPLETE" ]]; then
    echo "$1 stack has been created"
else
    aws cloudformation create-stack --stack-name "$1"-test-ecs-service --template-body file://service-cluster-alb.yaml --parameters ParameterKey=ParentVPCStack,ParameterValue=VPC ParameterKey=LoadBalancerPriority,ParameterValue=${RANDOM:0:4} ParameterKey=LoadBalancerHttps,ParameterValue=true ParameterKey=SubDomainNameWithDot,ParameterValue="$1". ParameterKey=Cpu,ParameterValue=1 ParameterKey=Memory,ParameterValue=2 ParameterKey=DesiredCount,ParameterValue=1 ParameterKey=MaxCapacity,ParameterValue=1  ParameterKey=AppImage,ParameterValue=nginx:latest ParameterKey=ParentZoneStack,ParameterValue=vpc-public-zone ParameterKey=ParentAlertStack,ParameterValue=vpc-alerts ParameterKey=ParentClusterStack,ParameterValue=test-tryorama-cluster ParameterKey=LoadBalancerHostPattern,ParameterValue="$1".holochain-aws.org ParameterKey=MinCapacity,ParameterValue=1 --capabilities CAPABILITY_IAM

    stackStatus=$(aws cloudformation wait stack-create-complete --stack-name "$1"-test-ecs-service) &
    spinner "Stack is being deployed"
    echo "///////////////"
    echo "Stack deployed and should be available at stack: "$1"-test-ecs-service / url: $1.holochain-aws.org"
fi