#!/usr/bin/env bash

PRETTYNAME=${1}
LOWERNAME=${PRETTYNAME,,}
UPPERNAME=${PRETTYNAME^^}
VARNAME=${LOWERNAME^}


echo prettyname: ${PRETTYNAME}
echo lowername: ${LOWERNAME}
echo uppername: ${UPPERNAME}
echo varname: ${VARNAME}


find . -name "*.rs" -exec sed -i "s/productname/${LOWERNAME}/g" '{}' \;
find . -name "*.rs" -exec sed -i "s/PRODUCTNAME/${UPPERNAME}/g" '{}' \;
find . -name "*.rs" -exec sed -i "s/ProductName/${PRETTYNAME}/g" '{}' \;
find . -name "*.rs" -exec sed -i "s/Productname/${VARNAME}/g" '{}' \;

find . -name "*.yaml" -exec sed -i "s/productname/${LOWERNAME}/g" '{}' \;
find . -name "*.yaml" -exec sed -i "s/PRODUCTNAME/${UPPERNAME}/g" '{}' \;
find . -name "*.yaml" -exec sed -i "s/ProductName/${PRETTYNAME}/g" '{}' \;
find . -name "*.yaml" -exec sed -i "s/Productname/${VARNAME}/g" '{}' \;

find . -name "*.toml" -exec sed -i "s/productname/${LOWERNAME}/g" '{}' \;
find . -name "*.toml" -exec sed -i "s/PRODUCTNAME/${UPPERNAME}/g" '{}' \;
find . -name "*.toml" -exec sed -i "s/ProductName/${PRETTYNAME}/g" '{}' \;
find . -name "*.toml" -exec sed -i "s/Productname/${VARNAME}/g" '{}' \;

find . -name "*.yml" -exec sed -i "s/productname/${LOWERNAME}/g" '{}' \;
find . -name "*.yml" -exec sed -i "s/PRODUCTNAME/${UPPERNAME}/g" '{}' \;
find . -name "*.yml" -exec sed -i "s/ProductName/${PRETTYNAME}/g" '{}' \;
find . -name "*.yml" -exec sed -i "s/Productname/${VARNAME}/g" '{}' \;

sed -i "s/productname/${LOWERNAME}/g" 'docker/Dockerfile'
sed -i "s/PRODUCTNAME/${UPPERNAME}/g" 'docker/Dockerfile'
sed -i "s/ProductName/${PRETTYNAME}/g" 'docker/Dockerfile'
sed -i "s/Productname/${VARNAME}/g" 'docker/Dockerfile'

mv rust/operator-binary/src/stackable-productname-operator.rs rust/operator-binary/src/stackable-${LOWERNAME}-operator.rs
mv examples/simple-productname-cluster.yaml examples/simple-${LOWERNAME}-cluster.yaml

mkdir -p deploy/crd
