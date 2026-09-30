#!/bin/bash
set -euo pipefail

# Remove the deployed oxidauth release (migration plan 12).

if [ -z "$1" ]; then
    echo "Error: Environment argument required"
    echo "Usage: $0 <environment>"
    echo "Example: $0 staging"
    exit 1
fi

ENVIRONMENT=$1

# same context deploy.sh targets — uninstalling against whatever context
# happens to be current would silently target the wrong cluster
kubectl config use-context fbl-k3s

helm uninstall --namespace ${ENVIRONMENT}-oxidauth ${ENVIRONMENT}-oxidauth-oxidauth
