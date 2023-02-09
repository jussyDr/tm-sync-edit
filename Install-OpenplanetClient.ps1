$OpenplanetDirectory = $env:USERPROFILE + "\OpenplanetNext"

if (-Not (Test-Path -Path $OpenplanetDirectory -PathType "Container")) {
    Write-Error -Message ("Could not find Openplanet directory at: '" + $OpenplanetDirectory + "'") -Category "ObjectNotFound"
    Exit
}

$PluginsDirectory = $OpenplanetDirectory + "\Plugins"

if (-Not (Test-Path -Path $PluginsDirectory -PathType "Container")) {
    New-Item -Path $PluginsDirectory -ItemType "Directory"
}

$PluginDirectory = $PluginsDirectory + "\SyncEdit"

if (Test-Path -Path $PluginDirectory -PathType "Container") {
    Remove-Item -Path $PluginDirectory -Recurse
}

New-Item -Path $PluginDirectory -ItemType "Directory"
Copy-Item -Path "clients\openplanet\*" -Destination $PluginDirectory -Recurse

cargo build --release -p openplanet-client-lib

$LibDirectory = $OpenplanetDirectory + "\lib"

if (-Not (Test-Path -Path $LibDirectory -PathType "Container")) {
    New-Item -Path $LibDirectory -ItemType "Directory"
}

$LibFile = $LibDirectory + "\SyncEdit.dll"

if (Test-Path -Path $LibFile -PathType "Leaf") {
    Remove-Item -Path $LibFile
}

Copy-Item -Path "target\release\openplanet_client_lib.dll" -Destination $LibFile
