#!/bin/bash

# 获取当前脚本所在的目录
SCRIPT_DIR=$(dirname "$(realpath "$0")")

sh $SCRIPT_DIR/drop-schema.sh
sh $SCRIPT_DIR/create-schema.sh

