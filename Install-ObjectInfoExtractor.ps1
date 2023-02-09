$OpenplanetDirectory = $env:USERPROFILE + "\OpenplanetNext"

if (-Not (Test-Path -Path $OpenplanetDirectory -PathType "Container")) {
    Write-Error -Message ("Could not find Openplanet directory at: '" + $OpenplanetDirectory + "'") -Category "ObjectNotFound"
    Exit
}

$PluginsDirectory = $OpenplanetDirectory + "\Plugins"

if (-Not (Test-Path -Path $PluginsDirectory -PathType "Container")) {
    New-Item -Path $PluginsDirectory -ItemType "Directory"
}

$PluginDirectory = $PluginsDirectory + "\ObjectInfoExtractor"

if (Test-Path -Path $PluginDirectory -PathType "Container") {
    Remove-Item -Path $PluginDirectory -Recurse
}

New-Item -Path $PluginDirectory -ItemType "Directory"
Copy-Item -Path "tools\object-info-extractor\*" -Destination $PluginDirectory -Recurse
