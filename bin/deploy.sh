#!/bin/bash
set -euo pipefail

# Deploy the oxidauth stack to Kubernetes (migration plan 12).
#
#   bin/deploy.sh staging
#   bin/deploy.sh production      # asks for confirmation first
#
# Prereq: the gitignored values plaintext must exist — run
# `crypt-keeper decrypt` at the repo root to materialize
# devops/helm/values-<env>.yaml and src/oxidauth/helm/values-<env>.yaml
# from their committed .enc siblings.

if [ -z "$1" ]; then
    echo "Error: Environment argument required"
    echo "Usage: $0 <environment>"
    echo "Example: $0 staging"
    exit 1
fi

ENVIRONMENT=$1

cd "$(dirname "$0")/.."

if [[ "$ENVIRONMENT" == "production" ]]; then
    echo "You are about to deploy to production."
    read -p "Are you sure? " -n 1 -r
    echo

    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# fbl-k3s is the freshbrewlabs k3s cluster every FBL project deploys to
# (parkinglot/bin/deploy.sh uses the same context).
echo "switching to oxidauth kubectl context"
kubectl config use-context fbl-k3s

echo "helm upgrade --namespace $ENVIRONMENT-oxidauth --create-namespace --install $ENVIRONMENT-oxidauth-oxidauth . -f ../../../devops/helm/values-$ENVIRONMENT.yaml -f values-$ENVIRONMENT.yaml"
echo
pushd src/oxidauth/helm 1> /dev/null && \
  helm upgrade --namespace $ENVIRONMENT-oxidauth --create-namespace --install $ENVIRONMENT-oxidauth-oxidauth . -f ../../../devops/helm/values-$ENVIRONMENT.yaml -f values-$ENVIRONMENT.yaml && \
  popd 1> /dev/null
echo
