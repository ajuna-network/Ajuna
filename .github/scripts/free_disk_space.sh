#!/usr/bin/env bash

get_available_space() {
    echo $(df -a $1 | awk 'NR > 1 {avail+=$4} END {print avail}')
}

BEFORE_SPACE=$(get_available_space)

# REF: https://github.com/apache/flink/blob/master/tools/azure-pipelines/free_disk_space.sh
echo "Removing large packages"
sudo apt-get remove -y '^dotnet-.*'
sudo apt-get remove -y 'php.*'
sudo apt-get remove -y '^mongodb-.*'
sudo apt-get remove -y '^mysql-.*'
sudo apt-get remove -y \
    azure-cli \
    google-cloud-sdk \
    hhvm \
    google-chrome-stable \
    firefox \
    powershell \
    mono-devel \
    libgl1-mesa-dri
sudo apt-get autoremove -y
sudo apt-get clean

# REF: https://github.com/apache/flink/blob/master/tools/azure-pipelines/free_disk_space.sh
echo "Removing large directories"
sudo rm -rf /usr/share/dotnet
sudo rm -rf /usr/local/graalvm
sudo rm -rf /usr/local/.ghcup /opt/ghc
sudo rm -rf /usr/local/share/powershell
sudo rm -rf /usr/local/share/chromium
sudo rm -rf /usr/local/lib/android
sudo rm -rf /usr/local/lib/node_modules

# REF: https://github.com/actions/runner-images/issues/2875#issuecomment-1163392159
echo "Removing tool cache"
sudo rm -rf "$AGENT_TOOLSDIRECTORY"

echo "Removing swap storage"
sudo swapoff -a
sudo rm -f /mnt/swapfile

echo "Finding space freed up"
AFTER_SPACE=$(get_available_space)
printf "%'.f\n" $((AFTER_SPACE - BEFORE_SPACE))
