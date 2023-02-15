#!/usr/bin/env bash

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
sudo rm -rf /usr/local/.ghcup
sudo rm -rf /usr/local/share/powershell
sudo rm -rf /usr/local/share/chromium
sudo rm -rf /usr/local/lib/android
sudo rm -rf /usr/local/lib/node_modules
