$UserFolder = $env:USERPROFILE
$OpenplanetFolder = $UserFolder + "\OpenplanetNext"
$PluginsFolder = $OpenplanetFolder + "\Plugins"
$TargetFolder = $PluginsFolder + "\SyncEdit"

if (Test-Path -Path $TargetFolder) {
    Remove-Item -Path $TargetFolder -Recurse
}

New-Item -Path $TargetFolder -ItemType "directory"

$SourceFolder = $PSScriptRoot
$SourceFiles = $SourceFolder + "\*"

Copy-Item -Path $SourceFiles -Destination $TargetFolder -Recurse -Include @("info.toml", "src")

cargo build -p "tm-sync-edit-client-lib" --release

$LibraryPath = $SourceFolder + "\..\target\release\tm_sync_edit_client_lib.dll"
$LibraryFolder = $OpenplanetFolder + "\lib"
$LibDest = $LibraryFolder + "\SyncEdit.dll"

if (-Not (Test-Path -Path $LibraryFolder)) {
    New-Item -Path $LibraryFolder -ItemType "directory"
}

Copy-Item -Path $LibraryPath -Destination $LibDest

